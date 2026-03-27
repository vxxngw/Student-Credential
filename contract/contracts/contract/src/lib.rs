#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype,
    Address, Env, String, Symbol,
    symbol_short, log,
};

// ============================================================
// DATA TYPES
// ============================================================

/// Loại credential
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum CredentialType {
    Certificate,   // Chứng chỉ hoàn thành khóa học
    Degree,        // Bằng tốt nghiệp
    Achievement,   // Thành tích đặc biệt
}

/// Trạng thái credential
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum CredentialStatus {
    Active,   // Đang có hiệu lực
    Revoked,  // Đã bị thu hồi
}

/// Thông tin một credential
#[contracttype]
#[derive(Clone, Debug)]
pub struct Credential {
    pub id: u64,
    pub student: Address,
    pub course_name: String,
    pub credential_type: CredentialType,
    pub status: CredentialStatus,
    pub issued_at: u64,     // timestamp (ledger sequence)
    pub points: u32,        // điểm thưởng đi kèm khi cấp
}

// ============================================================
// STORAGE KEYS
// ============================================================

#[contracttype]
pub enum DataKey {
    Admin,                      // địa chỉ admin
    CredentialCount,            // tổng số credential đã cấp
    Credential(u64),            // credential theo ID
    StudentPoints(Address),     // tổng điểm của sinh viên
    StudentCredCount(Address),  // số credential của sinh viên
}

// ============================================================
// CONTRACT
// ============================================================

#[contract]
pub struct StudentCredentialContract;

#[contractimpl]
impl StudentCredentialContract {

    // ----------------------------------------------------------
    // INIT — gọi 1 lần duy nhất khi deploy
    // ----------------------------------------------------------

    /// Khởi tạo contract, set admin
    pub fn initialize(env: Env, admin: Address) {
        // Kiểm tra chưa initialize
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::CredentialCount, &0u64);

        log!(&env, "Contract initialized. Admin: {}", admin);
    }

    // ----------------------------------------------------------
    // ISSUE — cấp credential cho sinh viên
    // ----------------------------------------------------------

    /// Admin cấp credential cho sinh viên
    /// Trả về ID của credential vừa tạo
    pub fn issue_credential(
        env: Env,
        student: Address,
        course_name: String,
        credential_type: CredentialType,
        reward_points: u32,
    ) -> u64 {
        // Chỉ admin được gọi
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        // Tạo ID mới
        let count: u64 = env.storage().instance()
            .get(&DataKey::CredentialCount)
            .unwrap_or(0);
        let new_id = count + 1;

        // Tạo credential
        let credential = Credential {
            id: new_id,
            student: student.clone(),
            course_name: course_name.clone(),
            credential_type,
            status: CredentialStatus::Active,
            issued_at: env.ledger().sequence() as u64,
            points: reward_points,
        };

        // Lưu credential
        env.storage().instance().set(&DataKey::Credential(new_id), &credential);
        env.storage().instance().set(&DataKey::CredentialCount, &new_id);

        // Cộng điểm thưởng cho sinh viên
        let current_points: u32 = env.storage().instance()
            .get(&DataKey::StudentPoints(student.clone()))
            .unwrap_or(0);
        env.storage().instance().set(
            &DataKey::StudentPoints(student.clone()),
            &(current_points + reward_points),
        );

        // Cập nhật số credential của sinh viên
        let cred_count: u32 = env.storage().instance()
            .get(&DataKey::StudentCredCount(student.clone()))
            .unwrap_or(0);
        env.storage().instance().set(
            &DataKey::StudentCredCount(student.clone()),
            &(cred_count + 1),
        );

        // Emit event
        env.events().publish(
            (symbol_short!("issued"), student.clone()),
            (new_id, reward_points),
        );

        log!(&env, "Credential #{} issued to student. Points awarded: {}", new_id, reward_points);

        new_id
    }

    // ----------------------------------------------------------
    // VERIFY — xác minh credential
    // ----------------------------------------------------------

    /// Xác minh credential có hợp lệ không
    /// Trả về true nếu Active, false nếu Revoked hoặc không tồn tại
    pub fn verify_credential(env: Env, credential_id: u64) -> bool {
        let key = DataKey::Credential(credential_id);

        if !env.storage().instance().has(&key) {
            return false;
        }

        let credential: Credential = env.storage().instance().get(&key).unwrap();
        credential.status == CredentialStatus::Active
    }

    /// Lấy toàn bộ thông tin của một credential
    pub fn get_credential(env: Env, credential_id: u64) -> Credential {
        let key = DataKey::Credential(credential_id);
        if !env.storage().instance().has(&key) {
            panic!("Credential not found");
        }
        env.storage().instance().get(&key).unwrap()
    }

    // ----------------------------------------------------------
    // REVOKE — thu hồi credential
    // ----------------------------------------------------------

    /// Admin thu hồi credential (ví dụ: sinh viên gian lận)
    pub fn revoke_credential(env: Env, credential_id: u64) {
        // Chỉ admin được gọi
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let key = DataKey::Credential(credential_id);
        if !env.storage().instance().has(&key) {
            panic!("Credential not found");
        }

        let mut credential: Credential = env.storage().instance().get(&key).unwrap();

        if credential.status == CredentialStatus::Revoked {
            panic!("Credential already revoked");
        }

        // Thu hồi điểm thưởng đã cộng
        let current_points: u32 = env.storage().instance()
            .get(&DataKey::StudentPoints(credential.student.clone()))
            .unwrap_or(0);
        let new_points = current_points.saturating_sub(credential.points);
        env.storage().instance().set(
            &DataKey::StudentPoints(credential.student.clone()),
            &new_points,
        );

        // Cập nhật trạng thái
        credential.status = CredentialStatus::Revoked;
        env.storage().instance().set(&key, &credential);

        // Emit event
        env.events().publish(
            (symbol_short!("revoked"), credential.student.clone()),
            credential_id,
        );

        log!(&env, "Credential #{} has been revoked", credential_id);
    }

    // ----------------------------------------------------------
    // REWARDS — xem điểm thưởng
    // ----------------------------------------------------------

    /// Lấy tổng điểm thưởng của một sinh viên
    pub fn get_points(env: Env, student: Address) -> u32 {
        env.storage().instance()
            .get(&DataKey::StudentPoints(student))
            .unwrap_or(0)
    }

    /// Lấy số lượng credential của một sinh viên
    pub fn get_credential_count(env: Env, student: Address) -> u32 {
        env.storage().instance()
            .get(&DataKey::StudentCredCount(student))
            .unwrap_or(0)
    }

    // ----------------------------------------------------------
    // ADMIN UTILS
    // ----------------------------------------------------------

    /// Lấy địa chỉ admin hiện tại
    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    /// Tổng số credential đã cấp
    pub fn total_credentials(env: Env) -> u64 {
        env.storage().instance()
            .get(&DataKey::CredentialCount)
            .unwrap_or(0)
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation};
    use soroban_sdk::{vec, Address, Env, IntoVal};

    #[test]
    fn test_full_flow() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, StudentCredentialContract);
        let client = StudentCredentialContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let student = Address::generate(&env);

        // 1. Initialize
        client.initialize(&admin);
        assert_eq!(client.get_admin(), admin);

        // 2. Issue credential
        let course = String::from_str(&env, "Soroban Smart Contract Bootcamp");
        let cred_id = client.issue_credential(
            &student,
            &course,
            &CredentialType::Certificate,
            &100u32,
        );
        assert_eq!(cred_id, 1);

        // 3. Verify credential
        assert_eq!(client.verify_credential(&1u64), true);

        // 4. Check points
        assert_eq!(client.get_points(&student), 100u32);

        // 5. Revoke credential
        client.revoke_credential(&1u64);
        assert_eq!(client.verify_credential(&1u64), false);

        // 6. Points bị trừ sau khi revoke
        assert_eq!(client.get_points(&student), 0u32);

        println!("All tests passed!");
    }

    #[test]
    #[should_panic(expected = "Already initialized")]
    fn test_double_initialize() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, StudentCredentialContract);
        let client = StudentCredentialContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        client.initialize(&admin); // phải panic
    }
}
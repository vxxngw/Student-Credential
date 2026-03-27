#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype,
    Address, Env, String, Vec,
    symbol_short, log,
};

// ============================================================
// DATA TYPES
// ============================================================

/// Trạng thái của một payment request
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum RequestStatus {
    Pending,    // Chờ thanh toán
    Paid,       // Đã thanh toán
    Cancelled,  // Đã huỷ
}

/// Một payment request (yêu cầu thanh toán)
#[contracttype]
#[derive(Clone, Debug)]
pub struct PaymentRequest {
    pub id: u64,
    pub from: Address,      // người yêu cầu (merchant / bạn bè)
    pub to: Address,        // người cần trả
    pub amount: i128,       // số XLM (tính bằng stroops: 1 XLM = 10_000_000 stroops)
    pub note: String,       // mô tả (ví dụ: "Tiền cơm trưa")
    pub status: RequestStatus,
    pub created_at: u64,
}

/// Một giao dịch đã hoàn thành
#[contracttype]
#[derive(Clone, Debug)]
pub struct Transaction {
    pub id: u64,
    pub sender: Address,
    pub receiver: Address,
    pub amount: i128,
    pub note: String,
    pub timestamp: u64,
}

/// Một bill cần chia
#[contracttype]
#[derive(Clone, Debug)]
pub struct SplitBill {
    pub id: u64,
    pub creator: Address,           // người tạo bill
    pub total_amount: i128,         // tổng tiền
    pub note: String,               // mô tả (ví dụ: "Ăn tối sinh nhật")
    pub participants: Vec<Address>, // danh sách người tham gia
    pub amount_per_person: i128,    // tiền mỗi người phải trả
    pub paid_count: u32,            // số người đã trả
    pub created_at: u64,
}

// ============================================================
// STORAGE KEYS
// ============================================================

#[contracttype]
pub enum DataKey {
    // Counters
    TxCount,
    RequestCount,
    SplitCount,

    // Records
    Transaction(u64),
    Request(u64),
    Split(u64),

    // Per-user tracking
    UserTxCount(Address),     // số giao dịch của user
    UserLastTx(Address),      // ID giao dịch gần nhất
    SplitPaid(u64, Address),  // đã thanh toán split bill chưa
}

// ============================================================
// CONTRACT
// ============================================================

#[contract]
pub struct LocalPaymentContract;

#[contractimpl]
impl LocalPaymentContract {

    // ----------------------------------------------------------
    // SEND PAYMENT — chuyển tiền trực tiếp
    // ----------------------------------------------------------

    /// Gửi XLM trực tiếp cho người khác kèm ghi chú
    /// amount tính bằng stroops (1 XLM = 10_000_000)
    pub fn send_payment(
        env: Env,
        sender: Address,
        receiver: Address,
        amount: i128,
        note: String,
    ) -> u64 {
        // Yêu cầu chữ ký của người gửi
        sender.require_auth();

        if amount <= 0 {
            panic!("Amount must be greater than 0");
        }
        if sender == receiver {
            panic!("Cannot send to yourself");
        }

        // Tạo transaction record on-chain
        let tx_count: u64 = env.storage().instance()
            .get(&DataKey::TxCount).unwrap_or(0);
        let new_id = tx_count + 1;

        let tx = Transaction {
            id: new_id,
            sender: sender.clone(),
            receiver: receiver.clone(),
            amount,
            note: note.clone(),
            timestamp: env.ledger().sequence() as u64,
        };

        env.storage().instance().set(&DataKey::Transaction(new_id), &tx);
        env.storage().instance().set(&DataKey::TxCount, &new_id);

        // Cập nhật thống kê user
        let user_count: u64 = env.storage().instance()
            .get(&DataKey::UserTxCount(sender.clone())).unwrap_or(0);
        env.storage().instance().set(
            &DataKey::UserTxCount(sender.clone()),
            &(user_count + 1),
        );
        env.storage().instance().set(&DataKey::UserLastTx(sender.clone()), &new_id);

        // Emit event
        env.events().publish(
            (symbol_short!("sent"), sender.clone()),
            (receiver.clone(), amount, new_id),
        );

        log!(&env, "Payment #{}: {} stroops sent. Note: {}", new_id, amount, note);

        new_id
    }

    // ----------------------------------------------------------
    // REQUEST PAYMENT — yêu cầu thanh toán
    // ----------------------------------------------------------

    /// Tạo yêu cầu thanh toán gửi đến một người
    /// Ví dụ: merchant yêu cầu khách hàng trả tiền
    pub fn request_payment(
        env: Env,
        from: Address,    // người yêu cầu
        to: Address,      // người cần trả
        amount: i128,
        note: String,
    ) -> u64 {
        from.require_auth();

        if amount <= 0 {
            panic!("Amount must be greater than 0");
        }
        if from == to {
            panic!("Cannot request from yourself");
        }

        let req_count: u64 = env.storage().instance()
            .get(&DataKey::RequestCount).unwrap_or(0);
        let new_id = req_count + 1;

        let request = PaymentRequest {
            id: new_id,
            from: from.clone(),
            to: to.clone(),
            amount,
            note: note.clone(),
            status: RequestStatus::Pending,
            created_at: env.ledger().sequence() as u64,
        };

        env.storage().instance().set(&DataKey::Request(new_id), &request);
        env.storage().instance().set(&DataKey::RequestCount, &new_id);

        // Emit event để notify người nhận request
        env.events().publish(
            (symbol_short!("req"), to.clone()),
            (from.clone(), amount, new_id),
        );

        log!(&env, "Payment request #{} created: {} stroops. Note: {}", new_id, amount, note);

        new_id
    }

    /// Thanh toán một request đang Pending
    pub fn pay_request(env: Env, payer: Address, request_id: u64) {
        payer.require_auth();

        let key = DataKey::Request(request_id);
        if !env.storage().instance().has(&key) {
            panic!("Request not found");
        }

        let mut request: PaymentRequest = env.storage().instance().get(&key).unwrap();

        if request.to != payer {
            panic!("You are not the payer of this request");
        }
        if request.status != RequestStatus::Pending {
            panic!("Request is not pending");
        }

        // Cập nhật trạng thái request
        request.status = RequestStatus::Paid;
        env.storage().instance().set(&key, &request);

        // Ghi lại giao dịch vào lịch sử
        let tx_count: u64 = env.storage().instance()
            .get(&DataKey::TxCount).unwrap_or(0);
        let new_tx_id = tx_count + 1;

        let tx = Transaction {
            id: new_tx_id,
            sender: payer.clone(),
            receiver: request.from.clone(),
            amount: request.amount,
            note: request.note.clone(),
            timestamp: env.ledger().sequence() as u64,
        };

        env.storage().instance().set(&DataKey::Transaction(new_tx_id), &tx);
        env.storage().instance().set(&DataKey::TxCount, &new_tx_id);

        env.events().publish(
            (symbol_short!("paid"), payer.clone()),
            (request_id, new_tx_id),
        );

        log!(&env, "Request #{} paid. Tx #{}", request_id, new_tx_id);
    }

    /// Huỷ request — chỉ người tạo request mới huỷ được
    pub fn cancel_request(env: Env, caller: Address, request_id: u64) {
        caller.require_auth();

        let key = DataKey::Request(request_id);
        if !env.storage().instance().has(&key) {
            panic!("Request not found");
        }

        let mut request: PaymentRequest = env.storage().instance().get(&key).unwrap();

        if request.from != caller {
            panic!("Only the requester can cancel");
        }
        if request.status != RequestStatus::Pending {
            panic!("Request is not pending");
        }

        request.status = RequestStatus::Cancelled;
        env.storage().instance().set(&key, &request);

        log!(&env, "Request #{} cancelled", request_id);
    }

    // ----------------------------------------------------------
    // SPLIT BILL — chia bill nhóm
    // ----------------------------------------------------------

    /// Tạo bill chia đều cho một nhóm người
    /// Ví dụ: 4 người ăn tối, tổng 400k → mỗi người 100k
    pub fn create_split(
        env: Env,
        creator: Address,
        participants: Vec<Address>,
        total_amount: i128,
        note: String,
    ) -> u64 {
        creator.require_auth();

        let participant_count = participants.len();
        if participant_count < 2 {
            panic!("Need at least 2 participants");
        }
        if total_amount <= 0 {
            panic!("Total amount must be greater than 0");
        }

        let amount_per_person = total_amount / participant_count as i128;

        let split_count: u64 = env.storage().instance()
            .get(&DataKey::SplitCount).unwrap_or(0);
        let new_id = split_count + 1;

        let split = SplitBill {
            id: new_id,
            creator: creator.clone(),
            total_amount,
            note: note.clone(),
            participants: participants.clone(),
            amount_per_person,
            paid_count: 0,
            created_at: env.ledger().sequence() as u64,
        };

        env.storage().instance().set(&DataKey::Split(new_id), &split);
        env.storage().instance().set(&DataKey::SplitCount, &new_id);

        env.events().publish(
            (symbol_short!("split"), creator.clone()),
            (new_id, total_amount, participant_count),
        );

        log!(
            &env,
            "Split bill #{}: {} stroops / {} people = {} each. Note: {}",
            new_id, total_amount, participant_count, amount_per_person, note
        );

        new_id
    }

    /// Một người trong nhóm xác nhận đã trả phần của mình
    pub fn pay_split(env: Env, payer: Address, split_id: u64) {
        payer.require_auth();

        let key = DataKey::Split(split_id);
        if !env.storage().instance().has(&key) {
            panic!("Split bill not found");
        }

        let mut split: SplitBill = env.storage().instance().get(&key).unwrap();

        // Kiểm tra payer có trong danh sách không
        let mut is_participant = false;
        for p in split.participants.iter() {
            if p == payer {
                is_participant = true;
                break;
            }
        }
        if !is_participant {
            panic!("You are not a participant of this split");
        }

        // Kiểm tra đã trả chưa
        let paid_key = DataKey::SplitPaid(split_id, payer.clone());
        if env.storage().instance().has(&paid_key) {
            panic!("You already paid this split");
        }

        // Đánh dấu đã trả
        env.storage().instance().set(&paid_key, &true);
        split.paid_count += 1;
        env.storage().instance().set(&key, &split);

        env.events().publish(
            (symbol_short!("spaid"), payer.clone()),
            (split_id, split.amount_per_person),
        );

        log!(
            &env,
            "Split #{}: {} paid {} stroops. {}/{} paid",
            split_id, payer, split.amount_per_person,
            split.paid_count, split.participants.len()
        );
    }

    // ----------------------------------------------------------
    // READ — đọc dữ liệu / lịch sử
    // ----------------------------------------------------------

    /// Lấy thông tin một giao dịch theo ID
    pub fn get_transaction(env: Env, tx_id: u64) -> Transaction {
        let key = DataKey::Transaction(tx_id);
        if !env.storage().instance().has(&key) {
            panic!("Transaction not found");
        }
        env.storage().instance().get(&key).unwrap()
    }

    /// Lấy thông tin một payment request
    pub fn get_request(env: Env, request_id: u64) -> PaymentRequest {
        let key = DataKey::Request(request_id);
        if !env.storage().instance().has(&key) {
            panic!("Request not found");
        }
        env.storage().instance().get(&key).unwrap()
    }

    /// Lấy thông tin một split bill
    pub fn get_split(env: Env, split_id: u64) -> SplitBill {
        let key = DataKey::Split(split_id);
        if !env.storage().instance().has(&key) {
            panic!("Split not found");
        }
        env.storage().instance().get(&key).unwrap()
    }

    /// Kiểm tra một người đã trả split bill chưa
    pub fn is_split_paid(env: Env, split_id: u64, participant: Address) -> bool {
        env.storage().instance()
            .has(&DataKey::SplitPaid(split_id, participant))
    }

    /// Tổng số giao dịch của một user
    pub fn get_user_tx_count(env: Env, user: Address) -> u64 {
        env.storage().instance()
            .get(&DataKey::UserTxCount(user))
            .unwrap_or(0)
    }

    /// Tổng số giao dịch toàn hệ thống
    pub fn total_transactions(env: Env) -> u64 {
        env.storage().instance()
            .get(&DataKey::TxCount)
            .unwrap_or(0)
    }

    /// Tổng số split bill đã tạo
    pub fn total_splits(env: Env) -> u64 {
        env.storage().instance()
            .get(&DataKey::SplitCount)
            .unwrap_or(0)
    }

    /// Tổng số payment request đã tạo
    pub fn total_requests(env: Env) -> u64 {
        env.storage().instance()
            .get(&DataKey::RequestCount)
            .unwrap_or(0)
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{vec, Address, Env, String};

    #[test]
    fn test_send_payment() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, LocalPaymentContract);
        let client = LocalPaymentContractClient::new(&env, &contract_id);

        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);
        let note = String::from_str(&env, "Tien com trua");

        // Gửi 10 XLM = 100_000_000 stroops
        let tx_id = client.send_payment(&sender, &receiver, &100_000_000i128, &note);
        assert_eq!(tx_id, 1);

        let tx = client.get_transaction(&1u64);
        assert_eq!(tx.amount, 100_000_000i128);
        assert_eq!(tx.sender, sender);
        assert_eq!(tx.receiver, receiver);
        assert_eq!(client.total_transactions(), 1u64);
        assert_eq!(client.get_user_tx_count(&sender), 1u64);

        println!("test_send_payment passed!");
    }

    #[test]
    fn test_request_and_pay() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, LocalPaymentContract);
        let client = LocalPaymentContractClient::new(&env, &contract_id);

        let merchant = Address::generate(&env);
        let customer = Address::generate(&env);
        let note = String::from_str(&env, "Tien cafe");

        // Merchant tạo request 5 XLM
        let req_id = client.request_payment(
            &merchant, &customer, &50_000_000i128, &note,
        );
        assert_eq!(req_id, 1);

        let req = client.get_request(&1u64);
        assert_eq!(req.status, RequestStatus::Pending);

        // Customer thanh toán
        client.pay_request(&customer, &1u64);
        let req_after = client.get_request(&1u64);
        assert_eq!(req_after.status, RequestStatus::Paid);

        println!("test_request_and_pay passed!");
    }

    #[test]
    fn test_cancel_request() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, LocalPaymentContract);
        let client = LocalPaymentContractClient::new(&env, &contract_id);

        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let note = String::from_str(&env, "test");

        client.request_payment(&alice, &bob, &100_000_000i128, &note);
        client.cancel_request(&alice, &1u64);

        let req = client.get_request(&1u64);
        assert_eq!(req.status, RequestStatus::Cancelled);

        println!("test_cancel_request passed!");
    }

    #[test]
    fn test_split_bill() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, LocalPaymentContract);
        let client = LocalPaymentContractClient::new(&env, &contract_id);

        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let charlie = Address::generate(&env);

        let participants = vec![&env, alice.clone(), bob.clone(), charlie.clone()];
        let note = String::from_str(&env, "An toi sinh nhat");

        // 300 XLM cho 3 người → mỗi người 100 XLM
        let split_id = client.create_split(
            &alice, &participants, &3_000_000_000i128, &note,
        );
        assert_eq!(split_id, 1);

        let split = client.get_split(&1u64);
        assert_eq!(split.amount_per_person, 1_000_000_000i128);
        assert_eq!(split.paid_count, 0u32);

        // Bob trả phần của mình
        client.pay_split(&bob, &1u64);
        assert_eq!(client.is_split_paid(&1u64, &bob), true);
        assert_eq!(client.is_split_paid(&1u64, &alice), false);

        let split_after = client.get_split(&1u64);
        assert_eq!(split_after.paid_count, 1u32);

        println!("test_split_bill passed!");
    }

    #[test]
    #[should_panic(expected = "Cannot send to yourself")]
    fn test_cannot_send_to_self() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, LocalPaymentContract);
        let client = LocalPaymentContractClient::new(&env, &contract_id);
        let user = Address::generate(&env);
        let note = String::from_str(&env, "test");
        client.send_payment(&user, &user, &100i128, &note);
    }

    #[test]
    #[should_panic(expected = "You already paid this split")]
    fn test_double_pay_split() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, LocalPaymentContract);
        let client = LocalPaymentContractClient::new(&env, &contract_id);

        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let participants = vec![&env, alice.clone(), bob.clone()];
        let note = String::from_str(&env, "test");

        client.create_split(&alice, &participants, &200_000_000i128, &note);
        client.pay_split(&bob, &1u64);
        client.pay_split(&bob, &1u64); // phải panic
    }
}
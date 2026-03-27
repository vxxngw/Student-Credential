<img width="1048" height="819" alt="image" src="https://github.com/user-attachments/assets/a9f6fcf9-2250-4bd2-93d3-91e1115b66fb" /># 🎓 Student Credential & Reward System

> A decentralized credential management system built on Stellar Blockchain using Soroban Smart Contracts.

---

## 📖 Description

**Student Credential & Reward System** is a decentralized application (dApp) built on the **Stellar blockchain** that allows educational institutions to **issue, verify, and manage academic credentials** — such as certificates, degrees, and achievements — in a transparent, tamper-proof, and borderless manner.

### The Problem
Traditional academic credentials (paper certificates, PDFs) are easy to **forge, lose, or go unverified**. Employers and institutions often have no reliable way to verify a student's qualifications without contacting the issuing school directly — a slow and costly process.

### The Solution
By storing credentials **on-chain via Soroban smart contracts**, this system ensures that:
- Every credential is **permanently recorded** and cannot be altered
- Anyone can **instantly verify** a credential's authenticity with just an ID
- Students are **rewarded with points** for their academic achievements
- Institutions can **revoke** fraudulent or erroneous credentials in real-time

### Why Stellar?
Stellar was chosen for its **near-instant finality (3–5 seconds)**, **ultra-low fees (~0.00001 XLM per operation)**, and its mission to make financial and data infrastructure accessible globally — perfectly aligned with the goal of making education credentials universally accessible and verifiable.

---

## ✨ Features

### 🏛️ Issue Credential
- Admin (institution) can issue credentials to any student address
- Supports multiple credential types: **Certificate**, **Degree**, **Achievement**
- Each credential is assigned a unique on-chain ID
- Automatically awards **reward points** to the student upon issuance

### ✅ Verify Credential
- Anyone can verify whether a credential is **Active** or **Revoked**
- Returns full credential details: student address, course name, type, issue date, and points
- No permission required — fully public and transparent

### 🏆 Reward Points System
- Students accumulate points each time a credential is issued to them
- Points are automatically **deducted** if a credential is revoked
- Institutions can use points to **rank, reward, or unlock perks** for top students

### 🚫 Revoke Credential
- Admin can revoke any credential (e.g., in cases of academic fraud)
- Revocation is **instant and on-chain** — no ambiguity
- Associated reward points are automatically reclaimed

---

## 📄 Contract

**Network:** Stellar Testnet

**Contract Address:**
```
CCX7SBW2EKTB6ZBUCKUOM2EEEPOGAFSH37UCO7WJC22WIYGZHEZ53NVB
```

🔍 **View on Stellar Expert:**
[https://stellar.expert/explorer/testnet/contract/CCX7SBW2EKTB6ZBUCKUOM2EEEPOGAFSH37UCO7WJC22WIYGZHEZ53NVB](https://stellar.expert/explorer/testnet/contract/CCX7SBW2EKTB6ZBUCKUOM2EEEPOGAFSH37UCO7WJC22WIYGZHEZ53NVB)

### 📸 Contract Screenshot
<img width="1919" height="1076" alt="image" src="https://github.com/user-attachments/assets/18aecdff-7e4f-4225-aa34-14bb61850457" />



---

## 🚀 Future Scopes

This project was built in a single workshop session, but the vision goes far beyond a demo.

### Short-term
- **Frontend dApp** — a web interface where students can view their credentials and points using Freighter wallet
- **Multi-admin support** — allow multiple authorized issuers (professors, departments) instead of a single admin
- **Batch issuance** — issue credentials to an entire graduating class in one transaction

### Mid-term
- **NFT Credentials** — mint each credential as a unique Soroban-based token (similar to Soulbound Tokens), making them directly displayable on wallets and portfolios
- **Cross-institution network** — partner universities share a common verification layer so any employer worldwide can verify credentials from any member institution
- **Credential marketplace** — students can showcase verified credentials to potential employers or scholarship committees directly on-chain

### Long-term
- **AI-powered credential scoring** — integrate off-chain AI to analyze a student's credential portfolio and generate a trustworthy academic reputation score
- **Decentralized autonomous university (DAU)** — a fully on-chain governance system where students, professors, and alumni collectively vote on curriculum, credential standards, and institutional policies via XLM-weighted voting
- **Global education passport** — every student in the world has a single Stellar address that acts as their lifelong academic identity, recognized across borders without requiring any central authority

> *The long-term goal is a world where your education speaks for itself — instantly, globally, and without intermediaries.*

---

## 👤 Profile

| | |
|---|---|
| **Name** | *(Vo Quoc Vuong)* |
| **ID** | *(105701)* |
| **School** | *(Dong A University)* |
| **Major** | Information Technology — Blockchain |
| **Year** | 3rd Year |
| **Stellar Address** | *(GDCC77BMF7QSRW327XNBAGH4X7NKRGVGH7DGIMRDYKGJRY7JYE6BFVFD)* |

---

*Built with ❤️ at Stellar Blockchain Workshop · Powered by Soroban Smart Contracts*

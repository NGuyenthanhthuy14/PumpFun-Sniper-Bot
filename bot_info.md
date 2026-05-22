# 🤖 THÔNG TIN CẤU HÌNH CHI TIẾT CỦA CÁC BOT

Tài liệu này tổng hợp đầy đủ và chi tiết toàn bộ thông tin cấu hình, Token Telegram, Chat ID, Private Key, cơ sở dữ liệu và thông tin kết nối máy chủ VPS của các dự án Bot của bạn.

> [!WARNING]
> **BẢO MẬT CỰC KỲ QUAN TRỌNG:** File này chứa các thông tin nhạy cảm bao gồm **Telegram Bot Token**, **Private Key Solana** và **Mật khẩu VPS**. Tuyệt đối không chia sẻ file này công khai hoặc commit lên GitHub công cộng.

---

## 🟢 PHẦN 1: PUMPFUN SNIPER BOT (Workspace Hiện Tại)

Dưới đây là thông tin được trích xuất từ file `.env` và `Config.toml` của dự án **PumpFun-Sniper-Bot**:

### 1.1. Thông Tin Telegram & Ví Solana
| Tham số | Giá trị cấu hình | Mô tả |
| :--- | :--- | :--- |
| **TG_BOT_TOKEN** | `8904120615:AAGYNA1SMkZcBX2qEjKpg174O5IMj-YJ7eA` | Token điều khiển Telegram Bot |
| **TG_CHAT_ID** | `5123702171` | ID Chat Telegram dùng để nhận thông báo / điều khiển |
| **Solana Pubkey** | `GYdRZNWe2hdSsSspJLyMNPThaTid6KTzAjKwsKGsem4m` | Địa chỉ ví Solana công khai của Bot (Wallet 2) |
| **Solana Private Key** | `5phpe7qUZfnLCsC1jdvWaWLX6nfo1MkrWf2xsamMEPo9x8wTwVbsH6jJWWQN9j9asjXZZFdmppyyFfL7oxsMdah` | Khóa bảo mật của ví dùng để ký các giao dịch Snipe |

### 1.2. Cấu Hình Kết Nối & Hạ Tầng (Infrastructure)
*   **RPC Endpoint:** `https://rpc.shyft.to?api_key=Bs3GIL0Q3NdkzFNc`
*   **gRPC Endpoint (Yellowstone Geyser):** `https://grpc.fra.shyft.to`
*   **gRPC Token:** `b3df13df-ed50-49ae-8c58-cf17d160f7f5`
*   **Helius API Key:** `fbf55156-1dd8-4093-8177-f847caf05d1c`

### 1.3. Cài Đặt Giao Dịch Chính (Config.toml)
*   **Dev Mode:** `false` (Chế độ Production)
*   **Buy Amount:** `0.05 SOL`
*   **Slippage Percent:** `30%`
*   **Zero Slot Fee:** `0.0001 SOL`
*   **Helius Fee:** `0.0003 SOL`
*   **Max Open Positions:** `10`
*   **Buy Cooldown:** `300 ms`
*   **Min SOL Balance required:** `0.05 SOL`

---

## 🔵 PHẦN 2: SNIPE MIGRATION BOT (Dự Án Di Trú & VPS)

Dưới đây là thông tin chi tiết trích xuất từ cấu hình VPS, file `.env`, `vps_credentials.md` và `vps_resources.json` của **Snipe-Migration-Bot**:

### 2.1. Thông Tin VPS & Kết Nối SSH
*   **Địa chỉ IP VPS:** `154.43.52.31`
*   **SSH Port:** `22`
*   **Tài khoản SSH:** `root`
*   **Mật khẩu SSH:** `v)pio=0NZBoP`
*   **Thư mục lưu bot trên VPS:** `/root/Snipe-Migration-Bot`
*   **Câu lệnh kết nối nhanh:**
    ```bash
    ssh root@154.43.52.31
    ```
*   **Thông số cấu hình VPS:**
    *   **Hệ điều hành:** Ubuntu 22.04 LTS
    *   **CPU:** 4 Cores
    *   **RAM:** 8 GB
    *   **Storage:** 80 GB
    *   **Tốc độ mạng:** 1 Gbps

### 2.2. Thông Tin Telegram & Ví Solana
| Tham số | Giá trị cấu hình | Ghi chú |
| :--- | :--- | :--- |
| **TELEGRAM_BOT_TOKEN** | `8623563315:AAELHdspKIN2O37l4H-xqBy8wwVKBW8u4e4` | Token điều khiển Telegram Bot di trú |
| **ALLOWED_USER_ID (Khách)** | `5123702171` | ID chat được quyền sử dụng bot |
| **ALLOWED_USER_ID (Sếp)** | `1838672927` | ID chat quản trị cao cấp |
| **Solana Private Key** | `5JvhZCF6dY7tynyBiDbG4jvQaS8VSZd2cU5Cg2n4d1seBVJk9WyTcLJ8LjajcYawbQsgevYdqBiRk1r7t4URPcpu` | Khóa bảo mật ví chính dùng để Snipe di trú |

### 2.3. Cấu Hình Cơ Sở Dữ Liệu PostgreSQL
*   **Database Host:** `localhost` (hoặc `postgres` nếu dùng Docker)
*   **Database Port:** `5433` (hoặc `5432` trong Docker)
*   **Database Name:** `sniper_db`
*   **Database User:** `sniper_user`
*   **Database Password:** `sniper_pass_123`
*   **Mật mã mã hóa Private Key (lưu trong DB):** `sniper_pass_123`

### 2.4. Cấu Hình Kết Nối & Hạ Tầng
*   **RPC Endpoint:** `https://rpc.shyft.to?api_key=Bs3GIL0Q3NdkzFNc`
*   **gRPC Endpoint (Yellowstone Geyser):** `https://grpc.fra.shyft.to`
*   **gRPC Token:** `b3df13df-ed50-49ae-8c58-cf17d160f7f5`
*   **Jito Endpoint (Block Engine):** `https://mainnet.block-engine.jito.wtf`

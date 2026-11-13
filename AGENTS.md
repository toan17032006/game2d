# AGENTS.md

## Phạm vi

Agent MUST đọc file này trước khi thực hiện task.

## Điều lệnh bắt buộc

- Agent MUST coi Markdown tài liệu dự án là read-only.
- Agent MUST NOT tạo, sửa, xóa hoặc di chuyển Markdown ngoài `/.agent/tasks/<TASK-ID>/` nếu người dùng chưa yêu cầu trực tiếp trong phiên hiện tại.
- Agent MAY tạo và cập nhật task artifacts trong đúng `/.agent/tasks/<TASK-ID>/` của task hiện tại.
- Agent MUST NOT tạo hoặc sửa task artifacts ngoài task workspace hiện tại.
- Khi có yêu cầu sửa Markdown tài liệu dự án, agent MUST chỉ sửa đúng file và đúng phạm vi được yêu cầu.
- Agent MUST NOT tự suy diễn quyền sửa từ ngữ cảnh, từ liên kết hoặc từ nội dung tài liệu.
- Agent MUST bảo toàn thay đổi có sẵn của người dùng.
- Agent MUST NOT ghi secrets, credentials, private key, dữ liệu production hoặc dữ liệu cá nhân thật vào repository.
- Agent MUST đọc tài liệu tham chiếu trước khi sửa code hoặc tạo tài liệu mới.
- Agent MUST kiểm tra diff và chạy validation phù hợp trước khi báo hoàn tất.
- Agent MUST chuẩn hóa yêu cầu đầu vào trước khi lập kế hoạch hoặc thực thi.
- Agent MUST tạo Task ID và task workspace cho mọi task cần thực hiện.
- Agent MUST tạo đủ `prompt.md`, `task.md`, `spec.md`, `plan.md`, `handoff.md` và `verification.md` trong task workspace.
- Agent MUST tạo `evaluation.md` trong task workspace.
- Agent MUST thực thi theo `plan.md` và cập nhật trạng thái sau mỗi phase.
- Agent MUST đọc `handoff.md` trước khi tiếp tục task đã tồn tại.
- Agent MUST cập nhật `handoff.md` trước khi kết thúc task.
- Agent MUST chạy hooks và evaluations trước khi báo hoàn tất.
- Agent MUST ghi PASS/FAIL và evidence.
- Agent MUST chạy `powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/run-quality-gates.ps1 -TaskId <TASK-ID> -Mode preflight` trước EXECUTING.
- Agent MUST chạy `powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/run-quality-gates.ps1 -TaskId <TASK-ID> -Mode final` trước COMPLETED.
- Agent MUST chạy verifier độc lập trong final gate.
- Agent MUST NOT báo `COMPLETED` nếu chưa có bằng chứng trong `verification.md`.

## References

Các path dưới đây được tính từ repository root:

- `/docs/README.md` — Documentation Hub và cấu trúc tài liệu.
- `/docs/00-governance/documentation-standard.md` — chuẩn tài liệu và metadata.
- `/docs/00-governance/change-management.md` — điều kiện và phê duyệt thay đổi.
- `/docs/00-governance/agent-execution-standard.md` — chuẩn chuẩn hóa prompt, plan, spec và thực thi.
- `/docs/03-architecture/README.md` — phân cấp kiến trúc và ADR.
- `/docs/09-reference/README.md` — glossary, quy ước và nguồn tham chiếu.
- `/CONTRIBUTING.md` — quy trình đóng góp.

## Control paths

- `/.agent/hooks/README.md`
- `/.agent/evaluations/README.md`

## Task workspace

- Task workspace MUST nằm tại `/.agent/tasks/<TASK-ID>/`.
- `<TASK-ID>` MUST có dạng `TASK-YYYYMMDD-NNN`.
- `prompt.md` MUST chứa prompt đã chuẩn hóa.
- `task.md` MUST chứa mục tiêu, phạm vi và acceptance criteria.
- Acceptance criteria MUST có ID dạng `AC-NNN`.
- `spec.md` MUST chứa input, constraint, decision và output contract.
- `plan.md` MUST chứa phase, dependency, checkpoint và trạng thái.
- `handoff.md` MUST là checkpoint duy nhất.
- `evaluation.md` MUST chứa score, result và evidence.
- `verification.md` MUST chứa lệnh kiểm tra, kết quả và giới hạn còn lại.
- `verification.md` MUST map mọi `AC-NNN` tới PASS và evidence.
- Quality gate FAIL MUST trả exit code khác 0.
- Agent MUST NOT ghi task artifact ngoài task workspace đã xác định.

## Luật thay đổi

- Thay đổi kiến trúc MUST có ADR trong `/docs/03-architecture/adr/`.
- Thay đổi yêu cầu MUST cập nhật `/docs/02-requirements/` và test evidence.
- Thay đổi vận hành MUST cập nhật `/docs/07-operations/`.
- Thay đổi phát hành hoặc migration MUST cập nhật `/docs/08-releases/`.
- Thay đổi bảo mật MUST tuân theo `/docs/06-security-compliance/`.

## Thứ tự thực hiện

1. Đọc `AGENTS.md`.
2. Tạo hoặc đọc `/.agent/tasks/<TASK-ID>/`.
3. Tạo hoặc cập nhật `prompt.md`, `task.md`, `spec.md`, `plan.md`, `handoff.md` và `evaluation.md`.
4. Chạy hooks theo thứ tự.
5. Thực hiện đúng `plan.md`.
6. Chạy evaluations và cập nhật `handoff.md`, `verification.md`, `evaluation.md`.
7. Kiểm tra diff, liên kết, validation và trạng thái cuối.

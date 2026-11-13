# Specification

- Task ID: `TASK-20261113-007`

## Inputs

- Đánh giá hiện trạng 72% và danh sách 5 lỗi thực tế do người dùng cung cấp.
- Nguyên tắc Lauren Tan: verification skill chạy sản phẩm thật, evals lặp lại, lỗi lặp thành lint/CI rule, paved path, formal verification cho subsystem quan trọng, chỉ scale sau khi tin một agent.
- Quy chuẩn tại `/AGENTS.md` và `/docs/00-governance/agent-execution-standard.md`.

## Confirmed facts

- `$ErrorActionPreference = 'Stop'` biến mọi dòng stderr của native command thành terminating error, nên cảnh báo Git làm hỏng boundary check.
- `git diff --check` đang quét cả `.agent/tasks/<TASK-ID>/logs/`, nơi output dạng bảng có padding cuối dòng.
- `verify-product.ps1` cũ chỉ kiểm syntax và link, sau đó ghi "baseline product health verified" khi không có command.
- Regression runner cũ gọi child process qua `2>&1`, nên stderr của con làm abort cả suite.
- Chưa có artifact máy đọc được cho product verification, chưa có lint rule, chưa có registry subsystem, chưa có model check lifecycle.

## Inferences

- Một runtime dùng chung (capture stdout/stderr riêng, chỉ exit code là fatal) loại bỏ cả lớp lỗi này thay vì vá từng script.
- Exemption có hạn cho task lịch sử là cách trung thực để không viết lại hồ sơ cũ, trong khi task mới bị chặn cứng.
- Model check khám phá toàn bộ đường đi tới `COMPLETED` là hình thức formal verification khả thi cho lifecycle nhỏ, không cần thêm toolchain ngoài.

## Constraints

- PowerShell 5.1 và PowerShell 7.
- Không tạo hoặc sửa Markdown tài liệu dự án ngoài task workspace.
- Không thêm dependency ngoài repository.
- Mọi failure mode phải trả exit code khác 0.
- Không ghi secret, credential hoặc dữ liệu production.

## Decisions

- Phân loại input: `DIRECT`.
- Gom logic native process vào `/.agent/scripts/lib/agent-runtime.ps1`.
- Task không khai báo product verification sẽ FAIL; chỉ task lịch sử có exemption ghi rõ trong `/.agent/scripts/product-verification-baseline.json` (có `reviewBy`).
- Trạng thái product verification chỉ nhận `VERIFIED`, `SKIPPED`, `LEGACY_EXEMPT`, `FAILED`; không có trạng thái "health verified" suy diễn.
- Lint rule LINT-001..LINT-004 codify các lỗi lặp đã gặp.
- Registry `/.agent/verification/subsystems.json` là verification skill theo subsystem; invariant thiếu check là FAIL.
- Model check lifecycle khám phá 48 đường đi, 5 đường tới `COMPLETED`, và replay tất định.

## Test Command
powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/run-regression-tests.ps1

## Output contract

- `/.agent/scripts/lib/agent-runtime.ps1` [NEW]
- `/.agent/scripts/product-verification-baseline.json` [NEW]
- `/.agent/scripts/test-agent-lint.ps1` [NEW]
- `/.agent/scripts/model-check-task-lifecycle.ps1` [NEW]
- `/.agent/scripts/verify-subsystem.ps1` [NEW]
- `/.agent/verification/subsystems.json` [NEW]
- `/.agent/scripts/check-doc-boundaries.ps1` [MODIFY]
- `/.agent/scripts/run-quality-gates.ps1` [MODIFY]
- `/.agent/scripts/verify-product.ps1` [MODIFY]
- `/.agent/scripts/run-regression-tests.ps1` [MODIFY]
- `/.agent/scripts/validate-task.ps1` [MODIFY]
- `/.agent/scripts/verify-task.ps1` [MODIFY]
- `/.github/workflows/quality-gates.yml` [MODIFY]

## References

- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
- `/.agent/evaluations/README.md`

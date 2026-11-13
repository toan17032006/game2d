# Prompt

## Input classification

`DIRECT`

## Normalized request

- Tra cứu hướng dẫn mới nhất của Lauren Tan (pstack, verification skill, evals lặp lại, lint/CI rule hóa lỗi lặp, paved path, formal verification, trust ladder).
- Đối chiếu hệ thống agent hiện tại (khoảng 72%) và nâng lên mức tương đương mô hình tham chiếu.
- Bắt buộc xử lý các lỗi thực tế đã nêu:
  - `check-doc-boundaries.ps1` gọi `git status`; cảnh báo stderr của Git bị PowerShell coi là lỗi terminating.
  - `run-regression-tests.ps1` dừng giữa chừng và để lại thư mục tạm.
  - `verify-product.ps1` ghi "baseline product health verified" khi không có test command.
  - Regression cases chưa đạt PASS trên môi trường hiện tại.
  - Thiếu product verification thật, formal verification và verification skill riêng cho subsystem.
- Bắt buộc `spec.md` khai báo product verification command cho task code.

## Sources

- https://x.com/poteto/article/2069824386283319343
- https://x.com/poteto/article/2094457600259842065
- https://cursor.com/compile/london

## References

- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`

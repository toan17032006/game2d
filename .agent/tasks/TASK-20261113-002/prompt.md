# Prompt

## Role

Repository documentation optimizer.

## Objective

Chuẩn hóa toàn bộ Markdown thành điều lệnh ngắn, rõ và bắt buộc đối với agent.

## Scope

Mọi file `*.md` trong repository.

## Constraints

- Dùng `MUST`, `MUST NOT`, `SHOULD` và trạng thái chuẩn.
- Giữ path tham chiếu từ repository root.
- Giữ nội dung cần thiết cho thực thi.
- Xóa diễn giải dư thừa.

## Acceptance criteria

- Mọi file Markdown có cấu trúc và References.
- Tài liệu điều khiển agent dùng điều lệnh cứng.
- Không còn hướng dẫn mơ hồ hoặc lặp ý.

## References

- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`

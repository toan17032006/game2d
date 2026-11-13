# Specification

## Inputs

- Agent rules hiện có.
- Task workspace hiện có.

## Confirmed facts

- Agent cần output sát acceptance criteria.
- Agent cần lớp kiểm thử và đánh giá.

## Constraints

- Dùng điều lệnh ngắn.
- Có PASS/FAIL.
- Không thêm hook phụ thuộc framework ứng dụng.

## Output contract

- `.agent/hooks/`
- `.agent/evaluations/`
- `evaluation.md` trong task template.
- Cập nhật `/AGENTS.md` và execution standard.

## References

- `/docs/00-governance/documentation-standard.md`

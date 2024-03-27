.align 8
.data
.global _app_count
_app_count:
.quad 2
.quad app_0_name
.quad app_0_start
.quad app_1_name
.quad app_1_start
.quad app_1_end

.data
app_0_name:
  .string "forktest"
.align 8
app_0_start:
  .incbin "../user/target/x86_64/release/forktest"
app_0_end:

.data
app_1_name:
  .string "user_shell"
.align 8
app_1_start:
  .incbin "../user/target/x86_64/release/user_shell"
app_1_end:

.align 8
.data
.global _app_count
_app_count:
.quad 3
.quad app_0_name
.quad app_0_start
.quad app_1_name
.quad app_1_start
.quad app_2_name
.quad app_2_start
.quad app_2_end

.data
app_0_name:
  .string "forktest"
.align 8
app_0_start:
  .incbin "../user/target/x86_64/release/forktest"
app_0_end:

.data
app_1_name:
  .string "power_7"
.align 8
app_1_start:
  .incbin "../user/target/x86_64/release/power_7"
app_1_end:

.data
app_2_name:
  .string "user_shell"
.align 8
app_2_start:
  .incbin "../user/target/x86_64/release/user_shell"
app_2_end:

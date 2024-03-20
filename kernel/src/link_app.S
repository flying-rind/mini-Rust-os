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
  .string "00power_3"
.align 8
app_0_start:
  .incbin "../user/target/x86_64/debug/00power_3"
app_0_end:

.data
app_1_name:
  .string "01power_5"
.align 8
app_1_start:
  .incbin "../user/target/x86_64/debug/01power_5"
app_1_end:

.data
app_2_name:
  .string "02power_7"
.align 8
app_2_start:
  .incbin "../user/target/x86_64/debug/02power_7"
app_2_end:

%struct.Bool_Bool = type { i32, [0 x i8] }

%struct.Bool_Bool_False = type { i32, %struct.siko_Tuple_ }

%struct.Bool_Bool_True = type { i32, %struct.siko_Tuple_ }

%struct.Int_Int = type { i64 }

%struct.siko_Tuple_ = type {  }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %unit_19 = alloca %struct.siko_Tuple_, align 4
   %call_18 = alloca %struct.siko_Tuple_, align 4
   %call_17 = alloca %struct.Bool_Bool, align 4
   %call_16 = alloca %struct.Bool_Bool, align 4
   %lit_14 = alloca %struct.Int_Int, align 8
   %lit_13 = alloca %struct.Int_Int, align 8
   %call_12 = alloca %struct.Bool_Bool, align 4
   %lit_10 = alloca %struct.Int_Int, align 8
   %lit_9 = alloca %struct.Int_Int, align 8
   %call_8 = alloca %struct.Int_Int, align 8
   %lit_6 = alloca %struct.Int_Int, align 8
   %lit_5 = alloca %struct.Int_Int, align 8
   %call_4 = alloca %struct.Int_Int, align 8
   %lit_2 = alloca %struct.Int_Int, align 8
   %lit_1 = alloca %struct.Int_Int, align 8
   %tmp_lit_1_1 = getelementptr inbounds %struct.Int_Int, ptr %lit_1, i32 0, i32 0
   store i64 6, ptr %tmp_lit_1_1, align 8
   %tmp_lit_2_1 = getelementptr inbounds %struct.Int_Int, ptr %lit_2, i32 0, i32 0
   store i64 5, ptr %tmp_lit_2_1, align 8
   call void @Int_Int_add(ptr %lit_2, ptr %lit_1, ptr %call_4)
   %tmp_lit_5_1 = getelementptr inbounds %struct.Int_Int, ptr %lit_5, i32 0, i32 0
   store i64 6, ptr %tmp_lit_5_1, align 8
   %tmp_lit_6_1 = getelementptr inbounds %struct.Int_Int, ptr %lit_6, i32 0, i32 0
   store i64 5, ptr %tmp_lit_6_1, align 8
   call void @Int_Int_sub(ptr %lit_6, ptr %lit_5, ptr %call_8)
   %tmp_lit_9_1 = getelementptr inbounds %struct.Int_Int, ptr %lit_9, i32 0, i32 0
   store i64 6, ptr %tmp_lit_9_1, align 8
   %tmp_lit_10_1 = getelementptr inbounds %struct.Int_Int, ptr %lit_10, i32 0, i32 0
   store i64 5, ptr %tmp_lit_10_1, align 8
   call void @Int_Int_eq(ptr %lit_10, ptr %lit_9, ptr %call_12)
   %tmp_lit_13_1 = getelementptr inbounds %struct.Int_Int, ptr %lit_13, i32 0, i32 0
   store i64 6, ptr %tmp_lit_13_1, align 8
   %tmp_lit_14_1 = getelementptr inbounds %struct.Int_Int, ptr %lit_14, i32 0, i32 0
   store i64 5, ptr %tmp_lit_14_1, align 8
   call void @Int_Int_lessThan(ptr %lit_14, ptr %lit_13, ptr %call_16)
   call void @Bool_Bool_True(ptr %call_17)
   call void @Std_Basic_Util_assert(ptr %call_17, ptr %call_18)
   call void @siko_Tuple_(ptr %unit_19)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %unit_19, i64 0, i1 false)
   ret void
}

define private void @Std_Basic_Util_assert(ptr noundef %v, ptr noundef %fn_result) {
block0:
   %unit_9 = alloca %struct.siko_Tuple_, align 4
   %unit_5 = alloca %struct.siko_Tuple_, align 4
   %call_4 = alloca %struct.siko_Tuple_, align 4
   %matchValue_13 = alloca %struct.siko_Tuple_, align 4
   %match_var_2 = alloca %struct.siko_Tuple_, align 4
   %valueRef_1 = alloca %struct.Bool_Bool, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %valueRef_1, ptr align 4 %v, i64 4, i1 false)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %matchValue_13, ptr align 4 %match_var_2, i64 0, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %matchValue_13, i64 0, i1 false)
   ret void
block2:
   %tmp_switch_var_block2_1 = getelementptr inbounds %struct.Bool_Bool, ptr %valueRef_1, i32 0, i32 0
   %tmp_switch_var_block2_2 = load i32, ptr %tmp_switch_var_block2_1, align 4
   switch i32 %tmp_switch_var_block2_2, label %block3 [
i32 1, label %block5
]

block3:
   br label %block4
block4:
   call void @Std_Basic_Util_siko_runtime_abort(ptr %call_4)
   call void @siko_Tuple_(ptr %unit_5)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_2, ptr align 4 %unit_5, i64 0, i1 false)
   br label %block1
block5:
   br label %block6
block6:
   call void @siko_Tuple_(ptr %unit_9)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_2, ptr align 4 %unit_9, i64 0, i1 false)
   br label %block1
}

declare void @Std_Basic_Util_siko_runtime_abort(ptr noundef %fn_result)

define private void @Bool_Bool_True(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Bool_Bool_True, align 4
   %tag = getelementptr inbounds %struct.Bool_Bool_True, ptr %this, i32 0, i32 0
   store i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Bool_Bool_True, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

declare void @Int_Int_add(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

declare void @Int_Int_eq(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

declare void @Int_Int_lessThan(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

declare void @Int_Int_sub(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}



%struct.siko_Tuple_ = type {  }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @Main_foo(ptr noundef %fn_result) {
block0:
   %b0i1 = alloca %struct.siko_Tuple_, align 4
   call void @siko_Tuple_(ptr %b0i1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b0i1, i64 0, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %b0i2 = alloca %struct.siko_Tuple_, align 4
   %b0i1 = alloca %struct.siko_Tuple_, align 4
   call void @Main_foo(ptr %b0i1)
   call void @siko_Tuple_(ptr %b0i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b0i2, i64 0, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}



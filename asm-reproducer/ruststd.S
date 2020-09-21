.section .text.__rdl_alloc
.globl __rdl_alloc
.type __rdl_alloc STT_FUNC
__rdl_alloc:
	cmpq   $0x0, __cxa_thread_atexit_impl@GOTPCREL(%rip)

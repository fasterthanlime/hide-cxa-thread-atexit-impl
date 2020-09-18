
## How?

Just `cargo build`, with Rust 1.46, and linking should fail:

```raw
  = note: /usr/bin/ld: BFD (GNU Binutils) 2.35 assertion fail /build/binutils/src/binutils-gdb/bfd/elflink.c:14788
          collect2: error: ld returned 1 exit status
```

This is the relevant assertion:

```c
// in `binutils/bfd/elflink.c`

/* Append a RELA relocation REL to section S in BFD.  */

void
elf_append_rela (bfd *abfd, asection *s, Elf_Internal_Rela *rel)
{
  const struct elf_backend_data *bed = get_elf_backend_data (abfd);
  bfd_byte *loc = s->contents + (s->reloc_count++ * bed->s->sizeof_rela);
  BFD_ASSERT (loc + bed->s->sizeof_rela <= s->contents + s->size); // that's the one
  bed->s->swap_reloca_out (abfd, rel, loc);
}
```

## Why???

The idea is to get a Rust `cdylib` to load on `dlopen` *and unload on `dlclose`*.

Since many parts of Rust's libstd register TLS destructors via `__cxa_thread_atexit_impl`,
any `cdylib` that, says, calls `println!`, or `std::thread::current()`, is un-unloadable.

There *is* a fallback in the Rust libstd, that doesn't rely on `__cxa_thread_atexit_impl`,
and the idea is to force libstd to use that fallback, by *hiding* glibc's `__cxa_thread_atexit_impl`.

That's the purpose of `src/stub.S` - we define a global symbol and set its value
to a constant `0x0`.

Then, that part of Rust's libstd code:

```rust
// in `rust/src/libstd/sys/unix/fast_thread_local.rs`

pub unsafe fn register_dtor(t: *mut u8, dtor: unsafe extern "C" fn(*mut u8)) {
    use crate::mem;
    use crate::sys_common::thread_local::register_dtor_fallback;

    extern "C" {
        #[linkage = "extern_weak"]
        static __dso_handle: *mut u8;
        #[linkage = "extern_weak"]
        static __cxa_thread_atexit_impl: *const libc::c_void;
    }
    if !__cxa_thread_atexit_impl.is_null() {
        type F = unsafe extern "C" fn(
            dtor: unsafe extern "C" fn(*mut u8),
            arg: *mut u8,
            dso_handle: *mut u8,
        ) -> libc::c_int;
        mem::transmute::<*const libc::c_void, F>(__cxa_thread_atexit_impl)(
            dtor,
            t,
            &__dso_handle as *const _ as *mut _,
        );
        return;
    }
    register_dtor_fallback(t, dtor);
}
```

...sees a null `__cxa_thread_atexit_impl`, and uses the fallback instead.

## Why not just patch Rust's libstd to use the fallback when you want to?

Well, I can't think of a good way to patch it right now, and it would take a few
weeks (minimum) to get through review and land in a stable release.

## Do you blame GNU ld?

Not at all - this is pretty cursed. When I tried reproducing this with a C project,
I stumbled upon a *proper* error message to the likes of "can't perform somesuch
relocation against constant symbol" and I'm assuming this is a variant of that.

So, I don't blame GNU ld for blowing up on this at all.

In fact, if the stub is built as a shared object instead, and `LD_PRELOAD` is used
to inject it, everything works fine. `ld` is happily ignoring all the cursedness,
and `rtld` does its job, looking up symbols, and finding it in `libstub.so` before
`libc.so.6` - and both the *host* Rust program and the Rust `cdylib` library use
the fallback instead.

But [Siddhesh](https://twitter.com/siddhesh_p/status/1306948850481991680) told me
an assertion failure is a bug, so, there it is!

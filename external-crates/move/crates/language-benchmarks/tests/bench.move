// `cargo bench` will call a public function in this module.
// The Module must be called `Bench` and the set of public functions are callable from the bench (Rust code).
// `benches/transaction.rs` contains the calling code.
// The idea is that you build your scenario with a public entry point and a bunch of private functions as needed.
module 0x6::Bench {
    use std::vector;

    //
    // Global helpers
    //
    fun check(check: bool, code: u64) {
        if (check) () else abort code
    }

    //
    // `arith` benchmark
    //
    public fun arith() {
        let i = 0;
        // 10000 is the number of loops to make the benchmark run for a couple of minutes, which is an eternity.
        // Adjust according to your needs, it's just a reference
        while (i < 10000) {
            1;
            10 + 3;
            10;
            7 + 5;
            let x = 1;
            let y = x + 3;
            check(x + y == 5, 10);
            i = i + 1;
        };
    }

    //
    // `call` benchmark
    //
    public fun call() {
        let i = 0;
        // 3000 is the number of loops to make the benchmark run for a couple of minutes, which is an eternity.
        // Adjust according to your needs, it's just a reference
        while (i < 3000) {
            let b = call_1(@0x0, 128);
            call_2(b);
            i = i + 1;
        };
    }

    fun call_1(addr: address, val: u64): bool {
        let b = call_1_1(&addr);
        call_1_2(val, val);
        b
    }

    fun call_1_1(_addr: &address): bool {
        true
    }

    fun call_1_2(val1: u64, val2: u64): bool {
        val1 == val2
    }

    fun call_2(b: bool) {
        call_2_1(b);
        check(call_2_2() == 400, 200);
    }

    fun call_2_1(b: bool) {
        check(b == b, 100)
    }

    fun call_2_2(): u64 {
        100 + 300
    }

    //
    // `natives` benchmark
    //
    fun test_vector_ops<T>(x1: T, x2: T): (T, T) {
        let v: vector<T> = vector::empty();
        check(vector::length(&v) == 0, 100);
        vector::push_back(&mut v, x1);
        check(vector::length(&v) == 1, 101);
        vector::push_back(&mut v, x2);
        check(vector::length(&v) == 2, 102);
        vector::swap(&mut v, 0, 1);
        x1 = vector::pop_back(&mut v);
        check(vector::length(&v) == 1, 103);
        x2 = vector::pop_back(&mut v);
        check(vector::length(&v) == 0, 104);
        vector::destroy_empty(v);
        (x1, x2)
    }

    fun test_vector() {
        test_vector_ops<u8>(1u8, 2u8);
        test_vector_ops<u64>(1u64, 2u64);
        test_vector_ops<u128>(1u128, 2u128);
        test_vector_ops<bool>(true, false);
        test_vector_ops<address>(@0x1, @0x2);
        test_vector_ops<vector<u8>>(vector::empty(), vector::empty());
    }

    public fun natives() {
        let i = 0;
        // 300 is the number of loops to make the benchmark run for a couple of minutes, which is an eternity.
        // Adjust according to your needs, it's just a reference
        while (i < 300) {
            test_vector();
            i = i + 1;
        }
    }
}

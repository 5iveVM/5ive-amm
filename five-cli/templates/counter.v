mut count: u64;

init {
    count = 0;
}

pub increment() {
    count = count + 1;
}

pub decrement() {
    count = count - 1;
}

pub reset() {
    count = 0;
}

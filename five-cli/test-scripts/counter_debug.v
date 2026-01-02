// Counter Debug Script
// Tests account state persistence with @init

account Counter {
    count: u64;
}

pub initialize(
    counter: Counter @mut @init(payer=payer) @signer,
    payer: account @signer
) {
    counter.count = 42;
}

pub increment(
    counter: Counter @mut
) {
    counter.count = counter.count + 1;
}

pub get_count(
    counter: Counter
) -> u64 {
    return counter.count;
}

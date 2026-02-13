pub test(formal: bool) -> string {
    let short_msg = "hi";
    let long_msg = "hello";
    if (formal) {
        return long_msg;
    }
    return short_msg;
}

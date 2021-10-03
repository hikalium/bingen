#[cfg(test)]
mod tests {
    use bingen::bingen;
    #[test]
    fn mrs_x0_dbgtr_el0() {
        let bin = bingen!("aarch64-linux-eabi", "mrs x0, DBGDTR_EL0");
        assert_eq!(bin, [0, 4, 51, 213]);
    }
    #[test]
    fn mov_r0_r1() {
        let bin = bingen!("aarch64-linux-eabi", "mov x0, 40");
        assert_eq!(bin, [0, 5, 128, 210]);
    }
    #[test]
    fn arm_mov_r0_r1() {
        let bin = bingen!("arm-linux-eabi", "mov r0, r1");
        assert_eq!(bin, [1, 0, 160, 225]);
    }
    #[test]
    fn x86_64_unknown_() {
        let bin = bingen!("x86_64-unknown-linux-gnu", "xorl %eax, %eax");
        assert_eq!(bin, [49, 192]);
    }
}

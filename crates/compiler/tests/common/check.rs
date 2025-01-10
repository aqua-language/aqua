#[macro_export]
macro_rules! check {
    ($a:expr, $msg:literal) => {{
        let msg = indoc::indoc!($msg);
        assert!(
            $a.msg == msg,
            "{}",
            common::check::diff($a.msg, msg.to_string())
        );
    }};
    ($a:expr, $b:expr) => {
        assert!($a == $b, "{}", {
            let a_str = format!("{}", $a);
            let b_str = format!("{}", $b);
            if a_str != b_str {
                common::check::diff(a_str, b_str)
            } else {
                let a_str = format!("{}", $a.verbose());
                let b_str = format!("{}", $b.verbose());
                if a_str != b_str {
                    common::check::diff(a_str, b_str)
                } else {
                    let a_str = format!("{:#?}", $a);
                    let b_str = format!("{:#?}", $b);
                    common::check::diff(a_str, b_str)
                }
            }
        });
    };
    ($a:expr, $b:expr, $msg:literal) => {{
        let msg = indoc::indoc!($msg);
        check!($a.val, $b);
        assert!(
            $a.msg == msg,
            "{}",
            common::check::diff($a.msg, msg.to_string())
        );
    }};
    (@value; $a:expr, $b:expr) => {{
        let a_str = format!("{:#?}", $a);
        let b_str = format!("{:#?}", $b);
        assert!($a == $b, "{}", common::check::diff(a_str, b_str));
    }};
}

#[allow(unused)]
pub fn diff(a: String, b: String) -> String {
    let mut output = String::new();
    let diff = similar::TextDiff::from_lines(&a, &b);
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            similar::ChangeTag::Delete => "A ",
            similar::ChangeTag::Insert => "B ",
            similar::ChangeTag::Equal => "  ",
        };
        output.push_str(&format!("{}{}", sign, change));
    }
    output
}

pub(crate) mod filter;
pub(crate) mod queryable;
pub(crate) mod querybuilder;
pub(crate) mod stream;

struct A<Ret, FN>
where
    FN: Fn(&i32) -> Ret,

{
    f: FN,
}

impl<Ret, FN> A<Ret, FN>
where
    FN: Fn(&i32) -> Ret,

{
    fn change<NewRet, FNN: Fn(&i32) -> NewRet>(self, newfn: FNN) -> A<NewRet, FNN> {
        A{f: newfn}
    }

    fn calling(self) -> i32{
        999
    }
}

fn new<Ret, FN>(f: FN) -> A<Ret, FN>
where
    FN: Fn(&i32) -> Ret,
{
    A { f }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a() {
        let bb = new(|x| 5);



        // bb = bb.change(|x| "".to_string());
        dbg!(bb.change(|x| "".to_string()).calling());
    }
}

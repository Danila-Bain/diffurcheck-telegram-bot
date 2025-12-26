pub struct Coef(pub i32);
impl std::fmt::Display for Coef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Coef(1) => {
                if f.sign_plus() {
                    write!(f, "+")
                } else {
                    write!(f, "")
                }
            }
            Coef(-1) => {
                write!(f, "-")
            }

            Coef(pos) if pos >= &0 => {
                if f.sign_plus() {
                    write!(f, "+ {pos}")
                } else {
                    write!(f, "{pos}")
                }
            }
            Coef(neg) => {
                write!(f, "- {}", neg.abs())
            }
        }
    }
}

pub struct Term(pub Coef, pub &'static str);
impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Term(Coef(coef), var) = *self;
        if var == "" {
            match coef {
                0 => {
                    write!(f, "")
                }
                1 => {
                    if f.sign_plus() {
                        write!(f, "+ 1")
                    } else {
                        write!(f, "1")
                    }
                }
                -1 => {
                    write!(f, "- 1")
                }
                c => {
                    if f.sign_plus() {
                        write!(f, "{:+}", Coef(c))
                    } else {
                        write!(f, "{}", Coef(c))
                    }
                }
            }
        } else {
            match coef {
                0 => {
                    write!(f, "")
                }
                c => {
                    if f.sign_plus() {
                        write!(f, "{:+} {}", Coef(c), var)
                    } else {
                        write!(f, "{} {}", Coef(c), var)
                    }
                }
            }
        }
    }
}
pub struct Exp(pub Coef, pub &'static str);
impl std::fmt::Display for Exp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Exp(Coef(coef), var) = *self;
        match coef {
            0 => {
                write!(f, "")
            }
            1 => {
                write!(f, "e^{var}")
            }
            c => {
                write!(f, "e^({} {})", Coef(c), var)
            }
        }
    }
}
pub struct Cos(pub Coef, pub &'static str);
impl std::fmt::Display for Cos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Cos(Coef(coef), var) = *self;
        match coef {
            0 => {
                write!(f, "")
            }
            1 => {
                write!(f, "cos {var}")
            }
            c => {
                write!(f, "cos({} {})", Coef(c), var)
            }
        }
    }
}
pub struct Sin(pub Coef, pub &'static str);
impl std::fmt::Display for Sin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Sin(Coef(coef), var) = *self;
        match coef {
            0 => {
                write!(f, "0")
            }
            1 => {
                write!(f, "sin {var}")
            }
            c => {
                write!(f, "sin({} {})", Coef(c), var)
            }
        }
    }
}

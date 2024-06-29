/// A commutative [monoid](https://en.wikipedia.org/wiki/Monoid), together with a function that lifts values of type `LiftingFrom` into the universe of the monoid. See the [range-based set reconciliation paper](https://github.com/AljoschaMeyer/rbsr_short/blob/main/main.pdf) for more context.
pub trait LiftingCommutativeMonoid<LiftingFrom>: Sized + Eq {
    /// The neutral element of the monoid.
    const NEUTRAL: Self;

    /// Lift a value into the monoid.
    fn lift(val: &LiftingFrom) -> Self;

    /// Combine two monoidal values. This function must be associative, commutative, and [`Self::NEUTRAL`] must be the neutral element of this function.
    fn combine(a: &Self, b: &Self) -> Self;
}

/// The trivial monoid that performs no computation. Use this when you *have* to supply a monoid but you do not actually need one.
impl<T> LiftingCommutativeMonoid<T> for () {
    const NEUTRAL: Self = ();

    fn lift(_val: &T) -> Self {
        return ();
    }

    fn combine(_a: &Self, _b: &Self) -> Self {
        return ();
    }
}

/// The `LiftingCommutativeMonoid` implementation for `usize` performs counting: any value is lifted to `1`, and `combine` is addition.
impl<T> LiftingCommutativeMonoid<T> for usize {
    const NEUTRAL: Self = 0;

    fn lift(_val: &T) -> Self {
        return 1;
    }

    fn combine(a: &Self, b: &Self) -> Self {
        return *a + *b;
    }
}
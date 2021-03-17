use ndarray::{IntoNdProducer, AssignElem};
use ndarray::{s, Array2};
use ndarray::Zip;
use num_traits::Num;

// This function clones elements from the first input to the second;
// the two producers must have the same shape
pub fn assign_to<'a, P1, P2, A>(from: P1, to: P2)
    where P1: IntoNdProducer<Item = &'a A>,
          P2: IntoNdProducer<Dim = P1::Dim>,
          P2::Item: AssignElem<A>,
          A: Clone + 'a
{
    Zip::from(from)
        .apply_assign_into(to, A::clone);
}

pub fn shift_by_row<T: Num + Clone>(a: &Array2<T>) -> Array2<T> {
    let mut b = Array2::zeros(a.dim());
    assign_to(a.slice(s![1.., ..]), b.slice_mut(s![..-1, ..]));
    b
}

pub fn shift_by_col<T: Num + Clone>(a: &Array2<T>) -> Array2<T> {
    let mut b = Array2::zeros(a.dim());
    assign_to(a.slice(s![.., 1..]), b.slice_mut(s![.., ..-1]));
    b
}

pub fn binarise_array<T: Num + Clone>(a : &Array2<T>) -> Array2<u8> {
    a.mapv(|x| (!x.is_zero()) as u8)
}

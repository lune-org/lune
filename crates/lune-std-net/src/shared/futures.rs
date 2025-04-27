use futures_lite::prelude::*;

pub use http_body_util::Either;

/**
    Combines the left and right futures into a single future
    that resolves to either the left or right output.

    This combinator is biased - if both futures resolve at
    the same time, the left future's output is returned.
*/
pub fn either<L: Future, R: Future>(
    left: L,
    right: R,
) -> impl Future<Output = Either<L::Output, R::Output>> {
    let fut_left = async move { Either::Left(left.await) };
    let fut_right = async move { Either::Right(right.await) };
    fut_left.or(fut_right)
}

pub mod post;

use post::*;

fn main() {
    let post = Post::<New>::new();

    let unpub = post.publish();
    let publ = unpub.allow();
    let del = publ.delete();
    // let del1 = unpub.deny(); // moved
}

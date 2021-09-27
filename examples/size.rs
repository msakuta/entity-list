use entity_list::EntityDynIter;

fn main() {
    println!("{}", std::mem::size_of::<EntityDynIter>())
}
struct KVPair<T> {
    key: i32,
    value: T,
}

struct Node<T> {
    keys: Vec<KVPair<T>>,
    children: Vec<Node<T>>,
}

#[cfg(test)]
mod tests {
    use super::*;
}

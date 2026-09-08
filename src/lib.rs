struct KVPair<T> {
    key: i32,
    value: T,
}

impl From<i32> for KVPair<()> {
    fn from(value: i32) -> Self {
        KVPair {
            key: value,
            value: (),
        }
    }
}

struct Node<T> {
    keys: Vec<KVPair<T>>,
    children: Vec<Node<T>>,
}

impl<T> Node<T> {
    fn busca_hijo(&self, key: i32) -> &Node<T> {
        let pos = self
            .keys
            .iter()
            .position(|kv| kv.key > key)
            .unwrap_or(self.keys.len())
            - 1;

        &self.children[pos]
    }

    fn busca(&self, key: i32) -> Option<&KVPair<T>> {
        if let Some(kv) = self.keys.iter().find(|kv| kv.key == key) {
            return Some(kv);
        }

        let hijo = self.busca_hijo(key);

        hijo.busca(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn busca_simple() {
        let arbol = Node {
            keys: vec![1.into(), 2.into(), 3.into()],
            children: vec![],
        };

        assert!(arbol.busca(1).is_some())
    }
}

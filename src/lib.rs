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
    fn key_pos(&self, key: i32) -> usize {
        self.keys
            .iter()
            .position(|kv| kv.key > key)
            .unwrap_or(self.keys.len())
    }

    fn busca_hijo(&self, key: i32) -> &Node<T> {
        &self.children[self.key_pos(key)]
    }

    fn es_hoja(&self) -> bool {
        self.children.len() == 0
    }

    fn busca(&self, key: i32) -> Option<&KVPair<T>> {
        if let Some(kv) = self.keys.iter().find(|kv| kv.key == key) {
            return Some(kv);
        }

        if self.es_hoja() {
            return None;
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

    #[test]
    fn busca_simple_no_existe() {
        let arbol = Node {
            keys: vec![1.into(), 2.into(), 3.into()],
            children: vec![],
        };

        assert!(arbol.busca(4).is_none())
    }

    #[test]
    fn busca_anidado() {
        let hoja1 = Node {
            keys: vec![1.into(), 2.into(), 3.into()],
            children: vec![],
        };

        let hoja2 = Node {
            keys: vec![6.into(), 8.into()],
            children: vec![],
        };

        let arbol = Node {
            keys: vec![4.into()],
            children: vec![hoja1, hoja2],
        };

        assert!(arbol.busca(1).is_some());
        assert!(arbol.busca(4).is_some());
        assert!(arbol.busca(6).is_some());
        assert!(arbol.busca(5).is_none());
    }
}

#[derive(PartialEq, Eq, Debug)]
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

    fn insertar(&mut self, kv: KVPair<T>) -> Result<(), KVPair<T>> {
        if self.es_hoja() {
            let pos = self.key_pos(kv.key);

            self.keys.insert(pos, kv);

            if self.keys.len() < 3 {
                return Ok(());
            }

            return Err(self.keys.remove(1));
        }

        let pos = self.key_pos(kv.key);

        let promovida = match self.children[pos].insertar(kv) {
            Ok(_) => {
                return Ok(());
            }
            Err(promovida) => promovida,
        };

        let node_to_split = self.children.remove(pos);

        let (left, right) = node_to_split.split();

        self.children.insert(pos, left);
        self.children.insert(pos + 1, right);

        self.keys.insert(pos, promovida);

        if self.keys.len() < 3 {
            return Ok(());
        }

        return Err(self.keys.remove(1));
    }

    fn split(mut self) -> (Node<T>, Node<T>) {
        if self.es_hoja() {
            assert!(self.keys.len() == 2);
            let right_key = self.keys.pop().unwrap();
            let left_key = self.keys.pop().unwrap();

            let left = Node {
                keys: vec![left_key],
                children: vec![],
            };

            let right = Node {
                keys: vec![right_key],
                children: vec![],
            };
            return (left, right);
        }

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn busca_simple() {
        let arbol = Node {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };

        assert!(arbol.busca(1).is_some())
    }

    #[test]
    fn busca_simple_no_existe() {
        let arbol = Node {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };

        assert!(arbol.busca(4).is_none())
    }

    #[test]
    fn busca_anidado() {
        let hoja1 = Node {
            keys: vec![1.into(), 2.into()],
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

    #[test]
    fn inserta_en_hoja() {
        let mut hoja = Node {
            keys: vec![1.into()],
            children: vec![],
        };

        assert!(hoja.insertar(2.into()).is_ok());
    }

    #[test]
    fn inserta_en_hoja_llena() {
        let mut hoja = Node {
            keys: vec![1.into(), 3.into()],
            children: vec![],
        };

        assert!(hoja.insertar(2.into()).is_err());

        assert_eq!(hoja.keys, vec![1.into(), 3.into()])
    }

    #[test]
    fn inserta_en_nodo_interno() {
        let hoja1 = Node {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };

        let hoja2 = Node {
            keys: vec![6.into()],
            children: vec![],
        };

        let mut arbol = Node {
            keys: vec![4.into()],
            children: vec![hoja1, hoja2],
        };

        assert!(arbol.insertar(7.into()).is_ok())
    }

    #[test]
    fn inserta_con_split_simple() {
        let hoja1 = Node {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };

        let hoja2 = Node {
            keys: vec![6.into()],
            children: vec![],
        };

        let mut arbol = Node {
            keys: vec![4.into()],
            children: vec![hoja1, hoja2],
        };

        assert!(arbol.insertar(3.into()).is_ok());
        assert!(arbol.keys.len() == 2);
        assert!(arbol.children.len() == 3);
    }
}

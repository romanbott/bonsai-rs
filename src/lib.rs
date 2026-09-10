use std::fmt;
use std::mem;

#[derive(PartialEq, Eq, Debug, Clone)]
struct KVPair<T: Clone> {
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

impl<T: fmt::Display + Clone> fmt::Display for KVPair<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.key, self.value)
    }
}

struct Node<T: Clone, const M: usize = 3> {
    keys: Vec<KVPair<T>>,
    children: Vec<Node<T, M>>,
}

impl<T: Clone, const M: usize> Node<T, M> {
    const Q: usize = (M + 1) / 2 - 1;

    fn key_pos(&self, key: i32) -> usize {
        self.keys
            .iter()
            .position(|kv| kv.key > key)
            .unwrap_or(self.keys.len())
    }

    fn busca_hijo(&self, key: i32) -> &Node<T, M> {
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

            if self.keys.len() < M {
                return Ok(());
            }

            return Err(self.keys.remove(M / 2));
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

        if self.keys.len() < M {
            return Ok(());
        }

        return Err(self.keys.remove(M / 2));
    }

    fn split(mut self) -> (Node<T, M>, Node<T, M>) {
        assert!(self.keys.len() == M - 1);

        if self.es_hoja() {
            let right_keys = self.keys.split_off(M / 2);

            let left = Node {
                keys: self.keys,
                children: vec![],
            };

            let right = Node {
                keys: right_keys,
                children: vec![],
            };
            return (left, right);
        }

        assert!(self.children.len() == M + 1);

        let right_children = self.children.split_off(M / 2 + 1);
        let right_keys = self.keys.split_off(M / 2);

        let left = Node {
            keys: self.keys,
            children: self.children,
        };

        let right = Node {
            keys: right_keys,
            children: right_children,
        };
        return (left, right);
    }

    fn elimina(&mut self, key: i32) {}
}

impl<T: fmt::Display + Clone, const M: usize> Node<T, M> {
    fn cabecera(&self) -> String {
        let ks = self
            .keys
            .iter()
            .map(|kv| kv.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!("[{ks}]")
    }

    fn hueco(&self, i: usize) -> (Option<i32>, Option<i32>) {
        let lo = if i > 0 {
            self.keys.get(i - 1).map(|kv| kv.key)
        } else {
            None
        };
        let hi = self.keys.get(i).map(|kv| kv.key);
        (lo, hi)
    }

    fn etiqueta_hijo(&self, lo: Option<i32>, hi: Option<i32>) -> String {
        let rango = match (lo, hi) {
            (None, None) => String::new(),
            (None, Some(h)) => format!("(<{h})"),
            (Some(l), None) => format!("(>{l})"),
            (Some(l), Some(h)) => format!("({l}..{h})"),
        };
        format!("{rango:<10}{}", self.cabecera())
    }

    fn imprime_hijos(&self, buf: &mut String, prefijo: &str) {
        let n = self.children.len();
        for (i, hijo) in self.children.iter().enumerate() {
            let ultimo = i + 1 == n;
            let conector = if ultimo { "└── " } else { "├── " };

            let (lo, hi) = self.hueco(i);
            buf.push_str(&format!(
                "{prefijo}{conector}{}\n",
                hijo.etiqueta_hijo(lo, hi)
            ));

            let pref = format!("{prefijo}{}", if ultimo { "    " } else { "│   " });
            hijo.imprime_hijos(buf, &pref);
        }
    }
}

struct ArbolB<T: Clone, const M: usize = 3> {
    root: Node<T, M>,
}

impl<T: fmt::Display + Clone, const M: usize> ArbolB<T, M> {
    fn a_cadena(&self) -> String {
        let mut buf = format!("{}\n", self.root.cabecera());
        self.root.imprime_hijos(&mut buf, "");
        buf
    }

    fn imprime(&self) {
        print!("{}", self.a_cadena());
    }
}

impl<T: Clone, const M: usize> ArbolB<T, M> {
    fn new() -> Self {
        assert!(M >= 3);
        Self {
            root: Node {
                keys: vec![],
                children: vec![],
            },
        }
    }

    fn insertar(&mut self, kv: KVPair<T>) {
        let promovida = match self.root.insertar(kv) {
            Ok(_) => {
                return;
            }
            Err(promovida) => promovida,
        };

        let new_root = Node {
            keys: vec![promovida],
            children: vec![],
        };

        let old_root = mem::replace(&mut self.root, new_root);

        let (left, right) = old_root.split();

        self.root.children.push(left);
        self.root.children.push(right);
    }

    fn es_valido(&self) -> bool {
        fn check<T: Clone, const M: usize>(nodo: &Node<T, M>) -> bool {
            if !nodo.es_hoja() && nodo.children.len() != nodo.keys.len() + 1 {
                return false;
            }
            nodo.children.iter().all(check)
        }
        check(&self.root)
    }

    fn hojas_balanceadas(&self) -> bool {
        fn profundidades<T: Clone, const M: usize>(nodo: &Node<T, M>, prof: usize) -> Vec<usize> {
            if nodo.es_hoja() {
                vec![prof]
            } else {
                nodo.children
                    .iter()
                    .flat_map(|h| profundidades(h, prof + 1))
                    .collect()
            }
        }
        let profs = profundidades(&self.root, 0);
        profs.iter().all(|&p| p == profs[0])
    }

    fn busca(&self, key: i32) -> Option<&KVPair<T>> {
        self.root.busca(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Nodo<T> = Node<T, 3>;
    type Arbol3<T> = ArbolB<T, 3>;

    #[test]
    fn busca_simple() {
        let arbol = Nodo {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };

        assert!(arbol.busca(1).is_some())
    }

    #[test]
    fn busca_simple_no_existe() {
        let arbol = Nodo {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };

        assert!(arbol.busca(4).is_none())
    }

    #[test]
    fn busca_anidado() {
        let hoja1 = Nodo {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };

        let hoja2 = Nodo {
            keys: vec![6.into(), 8.into()],
            children: vec![],
        };

        let arbol = Nodo {
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
        let mut hoja = Nodo {
            keys: vec![1.into()],
            children: vec![],
        };

        assert!(hoja.insertar(2.into()).is_ok());
    }

    #[test]
    fn inserta_en_hoja_llena() {
        let mut hoja = Nodo {
            keys: vec![1.into(), 3.into()],
            children: vec![],
        };

        assert!(hoja.insertar(2.into()).is_err());

        assert_eq!(hoja.keys, vec![1.into(), 3.into()])
    }

    #[test]
    fn inserta_en_nodo_interno() {
        let hoja1 = Nodo {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };

        let hoja2 = Nodo {
            keys: vec![6.into()],
            children: vec![],
        };

        let mut arbol = Nodo {
            keys: vec![4.into()],
            children: vec![hoja1, hoja2],
        };

        assert!(arbol.insertar(7.into()).is_ok())
    }

    #[test]
    fn inserta_con_split_simple() {
        let hoja1 = Nodo {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };

        let hoja2 = Nodo {
            keys: vec![6.into()],
            children: vec![],
        };

        let mut arbol = Nodo {
            keys: vec![4.into()],
            children: vec![hoja1, hoja2],
        };

        assert!(arbol.insertar(3.into()).is_ok());
        assert!(arbol.keys.len() == 2);
        assert!(arbol.children.len() == 3);
    }

    #[test]
    fn inserta_con_split_nodo_interno() {
        let hoja1 = Nodo {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };

        let hoja2 = Nodo {
            keys: vec![6.into()],
            children: vec![],
        };

        let hoja3 = Nodo {
            keys: vec![12.into()],
            children: vec![],
        };

        let mut arbol = Nodo {
            keys: vec![4.into(), 10.into()],
            children: vec![hoja1, hoja2, hoja3],
        };

        assert_eq!(arbol.insertar(3.into()), Err(4.into()));
        assert!(arbol.keys.len() == 2);
        assert!(arbol.children.len() == 4);
    }

    #[test]
    fn inserta_en_arbol_con_split_nodo_interno() {
        let hoja1 = Nodo {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };

        let hoja2 = Nodo {
            keys: vec![6.into()],
            children: vec![],
        };

        let hoja3 = Nodo {
            keys: vec![12.into()],
            children: vec![],
        };

        let mut arbol = Arbol3 {
            root: Nodo {
                keys: vec![4.into(), 10.into()],
                children: vec![hoja1, hoja2, hoja3],
            },
        };

        arbol.insertar(3.into());
        assert!(arbol.root.keys.len() == 1);
        assert!(arbol.root.children.len() == 2);
    }

    #[test]
    fn imprime_con_intervalos() {
        let kv = |k: i32, v: char| KVPair { key: k, value: v };

        let hoja15 = Nodo {
            keys: vec![kv(15, 'c')],
            children: vec![],
        };
        let hoja25 = Nodo {
            keys: vec![kv(25, 'd')],
            children: vec![],
        };
        let nodo20 = Nodo {
            keys: vec![kv(20, 'b')],
            children: vec![hoja15, hoja25],
        };
        let hoja5 = Nodo {
            keys: vec![kv(5, 'a')],
            children: vec![],
        };
        let hoja40 = Nodo {
            keys: vec![kv(40, 'e')],
            children: vec![],
        };

        let arbol = Arbol3 {
            root: Nodo {
                keys: vec![kv(10, 'A'), kv(30, 'B')],
                children: vec![hoja5, nodo20, hoja40],
            },
        };

        let esperado = concat!(
            "[10: A, 30: B]\n",
            "├── (<10)     [5: a]\n",
            "├── (10..30)  [20: b]\n",
            "│   ├── (<20)     [15: c]\n",
            "│   └── (>20)     [25: d]\n",
            "└── (>30)     [40: e]\n",
        );

        assert_eq!(arbol.a_cadena(), esperado);
    }

    #[test]
    fn imprime_arbol_tras_insertar_no_panics() {
        let mut arbol = Arbol3::new();

        for k in [5, 3, 8, 1, 4, 7, 9, 2, 6, 0] {
            arbol.insertar(KVPair { key: k, value: 'x' });
            arbol.imprime();
        }
    }

    #[test]
    fn invariante_arbol_tras_insertar_muchos() {
        let mut arbol = Arbol3::new();

        for k in 0..=30 {
            arbol.insertar(KVPair { key: k, value: 'x' });
        }

        for k in 0..=30 {
            assert!(arbol.root.busca(k).is_some(), "llave {k} no encontrada");
        }
        assert!(arbol.root.busca(99).is_none());
        assert!(arbol.es_valido(), "invariante hijos = llaves + 1 violada");
        assert!(arbol.hojas_balanceadas(), "hojas a distinta profundidad");
    }

    #[test]
    fn split_m4_promueve_la_tercera_llave() {
        let mut arbol = ArbolB::<char, 4>::new();

        for k in [1, 2, 3, 4] {
            arbol.insertar(KVPair { key: k, value: 'x' });
        }

        assert_eq!(arbol.root.keys.len(), 1);
        assert_eq!(arbol.root.keys[0].key, 3);
        assert_eq!(arbol.root.children.len(), 2);
        assert_eq!(
            arbol.root.children[0].keys,
            vec![KVPair { key: 1, value: 'x' }, KVPair { key: 2, value: 'x' }]
        );
        assert_eq!(
            arbol.root.children[1].keys,
            vec![KVPair { key: 4, value: 'x' }]
        );
    }

    fn insertar_y_verificar<const M: usize>(n: i32) {
        let mut arbol = ArbolB::<char, M>::new();

        for k in 0..=n {
            arbol.insertar(KVPair { key: k, value: 'x' });
        }

        for k in 0..=n {
            assert!(arbol.busca(k).is_some(), "M={M}: llave {k} no encontrada");
        }
        assert!(arbol.busca(n + 1).is_none(), "M={M}: sobra una llave");
        assert!(
            arbol.es_valido(),
            "M={M}: se rompe invariante hijos = llaves + 1"
        );
        assert!(
            arbol.hojas_balanceadas(),
            "M={M}: hojas a distinta profundidad"
        );
    }

    #[test]
    fn invariantes_con_distintos_m() {
        insertar_y_verificar::<3>(30);
        insertar_y_verificar::<4>(60);
        insertar_y_verificar::<5>(60);
        insertar_y_verificar::<6>(60);
    }
}

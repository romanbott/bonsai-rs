use std::fmt;
use std::mem;

#[derive(PartialEq, Eq, Clone)]
struct KVPair<T: Clone> {
    key: i32,
    value: T,
}

impl From<i32> for KVPair<()> {
    fn from(value: i32) -> Self {
        KVPair { key: value, value: () }
    }
}

impl<T: fmt::Debug + Clone> fmt::Debug for KVPair<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:?}", self.key, self.value)
    }
}

#[derive(PartialEq, Eq, Debug)]
enum ElimError<T: Clone> {
    NoEncontrada,
    Underflow(KVPair<T>),
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

    fn min(&self) -> &KVPair<T> {
        if self.children.len() == 0 {
            self.keys.first().unwrap()
        } else {
            self.children.first().unwrap().min()
        }
    }

    fn max(&self) -> &KVPair<T> {
        if self.children.len() == 0 {
            self.keys.last().unwrap()
        } else {
            self.children.last().unwrap().max()
        }
    }

    fn elimina(&mut self, key: i32) -> Result<KVPair<T>, ElimError<T>> {
        // Caso base: nodo hoja
        if self.es_hoja() {
            return match self.keys.iter().position(|kv| kv.key == key) {
                // La llave no existe en el árbol
                None => Err(ElimError::NoEncontrada),
                Some(i) => {
                    let kv = self.keys.remove(i);

                    // Si el nodo quedó por debajo del mínimo de llaves, avisar al caller
                    if self.keys.len() < Self::Q {
                        Err(ElimError::Underflow(kv))
                    } else {
                        Ok(kv)
                    }
                }
            };
        }

        // Nodo interno: la llave a eliminar está en este nodo
        if let Some(i) = self.keys.iter().position(|kv| kv.key == key) {
            // Reemplazar la llave por su predecesor (máximo del subárbol izquierdo)
            let predecesor = self.children[i].max().clone();
            let pk = predecesor.key;
            let eliminada = mem::replace(&mut self.keys[i], predecesor);

            // Eliminar el predecesor (ahora duplicado) del subárbol izquierdo
            return match self.children[i].elimina(pk) {
                // Si el hijo quedó bajo el mínimo, el caller debe rebalancearlo
                Err(ElimError::Underflow(_)) => self.rebalancea(i, eliminada),
                // El resto de los casos no altera este nodo: ya se hizo el reemplazo
                _ => Ok(eliminada),
            };
        }

        // Nodo interno: la llave no está, bajar por el hijo que la contiene
        let pos = self.key_pos(key);

        match self.children[pos].elimina(key) {
            // Si el hijo quedó bajo el mínimo, el caller debe rebalancearlo
            Err(ElimError::Underflow(kv)) => self.rebalancea(pos, kv),
            // Propagar el resultado tal cual (encontrada/eliminada o no encontrada)
            otro => otro,
        }
    }

    fn rebalancea(&mut self, pos: usize, kv: KVPair<T>) -> Result<KVPair<T>, ElimError<T>> {
        if (pos > 0) && (self.children[pos - 1].keys.len() > Self::Q) {
            let key_from_sibling = self.children[pos - 1].keys.pop().unwrap();

            let parent = mem::replace(&mut self.keys[pos - 1], key_from_sibling);

            self.children.get_mut(pos).unwrap().keys.insert(0, parent);

            if let Some(node_from_sibling) = self.children[pos - 1].children.pop() {
                self.children
                    .get_mut(pos)
                    .unwrap()
                    .children
                    .insert(0, node_from_sibling);
            };

            return Ok(kv);
        }

        unimplemented!("rebalancea: reparación de underflow pendiente")
    }
}

impl<T: fmt::Debug + Clone, const M: usize> Node<T, M> {
    fn cabecera(&self) -> String {
        let ks = self
            .keys
            .iter()
            .map(|kv| format!("{kv:?}"))
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

impl<T: fmt::Debug + Clone, const M: usize> fmt::Debug for Node<T, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = format!("{}\n", self.cabecera());
        self.imprime_hijos(&mut buf, "");
        write!(f, "{}", buf.trim_end())
    }
}

struct ArbolB<T: Clone, const M: usize = 3> {
    root: Node<T, M>,
}

type MapaB = ArbolB<()>;

impl<T: fmt::Debug + Clone, const M: usize> fmt::Debug for ArbolB<T, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.root)
    }
}

impl<T: fmt::Debug + Clone, const M: usize> ArbolB<T, M> {
    fn a_cadena(&self) -> String {
        format!("{:?}", self)
    }

    fn imprime(&self) {
        println!("{:?}", self);
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

    fn insertar(&mut self, clave: i32, valor: T) {
        let kv = KVPair {
            key: clave,
            value: valor,
        };

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

    fn elimina(&mut self, key: i32) -> Option<KVPair<T>> {
        let kv = match self.root.elimina(key) {
            Err(ElimError::NoEncontrada) => return None,
            Ok(kv) | Err(ElimError::Underflow(kv)) => kv,
        };

        if self.root.keys.is_empty() && self.root.children.len() == 1 {
            self.root = self.root.children.remove(0);
        }

        Some(kv)
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

        arbol.insertar(3, ());
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
            "[10: 'A', 30: 'B']\n",
            "├── (<10)     [5: 'a']\n",
            "├── (10..30)  [20: 'b']\n",
            "│   ├── (<20)     [15: 'c']\n",
            "│   └── (>20)     [25: 'd']\n",
            "└── (>30)     [40: 'e']",
        );

        assert_eq!(arbol.a_cadena(), esperado);
    }

    #[test]
    fn imprime_arbol_tras_insertar_no_panics() {
        let mut arbol = Arbol3::new();

        for k in [5, 3, 8, 1, 4, 7, 9, 2, 6, 0] {
            arbol.insertar(k, 'x');
            arbol.imprime();
        }
    }

    #[test]
    fn invariante_arbol_tras_insertar_muchos() {
        let mut arbol = Arbol3::new();

        for k in 0..=30 {
            arbol.insertar(k, 'x');
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
            arbol.insertar(k, 'x');
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
            arbol.insertar(k, 'x');
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

    #[test]
    fn elimina_de_hoja_sin_underflow() {
        let mut hoja = Nodo {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };

        assert_eq!(hoja.elimina(1), Ok(1.into()));
        assert_eq!(hoja.keys, vec![2.into()]);
    }

    #[test]
    fn elimina_de_hoja_retorna_underflow() {
        let mut hoja = Nodo {
            keys: vec![1.into()],
            children: vec![],
        };

        assert_eq!(hoja.elimina(1), Err(ElimError::Underflow(1.into())));
        assert!(hoja.keys.is_empty());
    }

    #[test]
    fn elimina_no_encontrada() {
        let mut hoja = Nodo {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };

        assert_eq!(hoja.elimina(5), Err(ElimError::NoEncontrada));
        assert_eq!(hoja.keys, vec![1.into(), 2.into()]);
    }

    #[test]
    fn elimina_de_nodo_interno() {
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

        assert_eq!(arbol.elimina(4), Ok(4.into()));
        assert_eq!(arbol.keys, vec![2.into()]);
        assert_eq!(arbol.children[0].keys, vec![1.into()]);
        assert_eq!(arbol.children[1].keys, vec![6.into()]);
    }

    #[test]
    fn elimina_desde_arbol() {
        let mut arbol = Arbol3::new();
        arbol.insertar(1, 'x');
        arbol.insertar(2, 'x');

        assert_eq!(arbol.elimina(99), None);

        assert_eq!(arbol.elimina(1), Some(KVPair { key: 1, value: 'x' }));
        assert!(arbol.busca(1).is_none());
        assert!(arbol.busca(2).is_some());

        assert_eq!(arbol.elimina(2), Some(KVPair { key: 2, value: 'x' }));
        assert!(arbol.busca(2).is_none());
        assert!(arbol.es_valido());
    }

    #[test]
    #[should_panic(expected = "rebalancea")]
    fn underflow_aun_no_implementado() {
        let mut arbol = Nodo {
            keys: vec![4.into()],
            children: vec![
                Nodo {
                    keys: vec![1.into()],
                    children: vec![],
                },
                Nodo {
                    keys: vec![6.into()],
                    children: vec![],
                },
            ],
        };

        let _ = arbol.elimina(1);
    }

    #[test]
    fn imprime_arbol_con_valor_unitario() {
        let mut arbol = ArbolB::<(), 4>::new();

        for k in [2, 1, 3, 4] {
            arbol.insertar(k, ());
        }

        let esperado = concat!(
            "[3: ()]\n",
            "├── (<3)      [1: (), 2: ()]\n",
            "└── (>3)      [4: ()]",
        );

        assert_eq!(arbol.a_cadena(), esperado);
    }

    #[test]
    fn rebalancea_prestamo_izquierdo_hojas() {
        let mut arbol = Nodo {
            keys: vec![10.into()],
            children: vec![
                Nodo {
                    keys: vec![1.into(), 5.into()],
                    children: vec![],
                },
                Nodo {
                    keys: vec![15.into()],
                    children: vec![],
                },
            ],
        };

        let res = arbol.elimina(15);
        assert_eq!(res, Ok(15.into()));

        assert_eq!(arbol.keys, vec![5.into()]);
        assert_eq!(arbol.children[0].keys, vec![1.into()]);
        assert_eq!(arbol.children[1].keys, vec![10.into()]);
    }

    #[test]
    fn rebalancea_prestamo_izquierdo_nodos_internos() {
        let mut arbol = Nodo {
            keys: vec![20.into()],
            children: vec![
                Nodo {
                    keys: vec![8.into(), 15.into()],
                    children: vec![
                        Nodo {
                            keys: vec![5.into()],
                            children: vec![],
                        },
                        Nodo {
                            keys: vec![10.into()],
                            children: vec![],
                        },
                        Nodo {
                            keys: vec![18.into()],
                            children: vec![],
                        },
                    ],
                },
                Nodo {
                    keys: vec![25.into()],
                    children: vec![
                        Nodo {
                            keys: vec![22.into()],
                            children: vec![],
                        },
                        Nodo {
                            keys: vec![28.into()],
                            children: vec![],
                        },
                    ],
                },
            ],
        };

        let res = arbol.rebalancea(1, 25.into());
        assert_eq!(res, Ok(25.into()));

        assert_eq!(arbol.keys, vec![15.into()]);

        assert_eq!(arbol.children[0].keys, vec![8.into()]);
        assert_eq!(arbol.children[0].children.len(), 2);

        assert_eq!(arbol.children[1].keys, vec![20.into(), 25.into()]);
        assert_eq!(arbol.children[1].children.len(), 3);
        assert_eq!(arbol.children[1].children[0].keys, vec![18.into()]);
    }
}

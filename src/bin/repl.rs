use bonsai_rs::{ArbolB, KVPair};
trait ArbolDyn {
    fn insertar(&mut self, k: i32);
    fn elimina(&mut self, k: i32) -> Option<KVPair<()>>;
    fn busca(&self, k: i32) -> bool;
    fn a_cadena(&self) -> String;
}

impl<const M: usize> ArbolDyn for ArbolB<(), M> {
    fn insertar(&mut self, k: i32) {
        self.insertar(k, ());
    }

    fn elimina(&mut self, k: i32) -> Option<KVPair<()>> {
        self.elimina(k)
    }

    fn busca(&self, k: i32) -> bool {
        self.busca(k).is_some()
    }

    fn a_cadena(&self) -> String {
        self.a_cadena_claves()
    }
}

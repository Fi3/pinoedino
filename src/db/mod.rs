// fake db just for POC

pub struct Db {
    referenced_id: std::collections::HashSet<u32>,
    referenced_tx: std::collections::HashMap<u32, Transaction>,
    clients: std::collections::HashMap<u16, crate::engine::Client>,
}

pub struct Transaction {
    pub parsed_tx: crate::engine::Transaction,
    pub client_id: u16,
}

impl Db {
    pub fn new() -> Self {
        Db {
            referenced_id: std::collections::HashSet::new(),
            referenced_tx: std::collections::HashMap::new(),
            clients: std::collections::HashMap::new(),
        }
    }

    pub fn has_id(&self, id: u32) -> bool {
        self.referenced_id.contains(&id)
    }

    pub fn add_id(&mut self, id: u32) {
        self.referenced_id.insert(id);
    }

    pub fn add_tx(&mut self, id: u32, tx: crate::engine::Transaction, client_id: u16) {
        let tx = Transaction {
            parsed_tx: tx,
            client_id,
        };
        self.referenced_tx.insert(id, tx);
    }

    pub fn add_client(&mut self, id: u16, client: crate::engine::Client) {
        self.clients.insert(id, client);
    }

    pub fn get_tx(&mut self, id: u32) -> Option<&Transaction> {
        self.referenced_tx.get(&id)
    }

    pub fn get_client(&mut self, id: u16) -> Option<&mut crate::engine::Client> {
        self.clients.get_mut(&id)
    }

    pub fn get_clients(&self) -> &std::collections::HashMap<u16, crate::engine::Client> {
        &self.clients
    }
}

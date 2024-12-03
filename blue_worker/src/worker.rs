
pub struct Worker {

}

pub enum WorkerState {
    //worker is sharing data to server
    Sharing(),
    //worker collecting info about closest devices
    Searching()
}

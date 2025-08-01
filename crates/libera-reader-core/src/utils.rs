pub(crate) enum MultiThreadTask {
  _ExtractImg,
  CalcHash,
}
impl MultiThreadTask {
  pub(crate) fn get_num_of_threads(&self) -> usize {
    let cpus = num_cpus::get();
    match self {
      MultiThreadTask::_ExtractImg => {
        if cpus >= 6 {
          cpus - 2
        } else if cpus == 1 {
          1
        } else {
          cpus - 1
        }
      }
      MultiThreadTask::CalcHash => 2,
    }
  }
}

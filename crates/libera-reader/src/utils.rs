pub(crate) enum _MultiThreadTask {
  _ExtractImg,
  CalcHash,
}
impl _MultiThreadTask {
  pub(crate) fn _get_num_of_threads(&self) -> usize {
    let cpus = num_cpus::get();
    match self {
      _MultiThreadTask::_ExtractImg => {
        if cpus >= 6 {
          cpus - 2
        } else if cpus == 1 {
          1
        } else {
          cpus - 1
        }
      }
      _MultiThreadTask::CalcHash => 2,
    }
  }
}

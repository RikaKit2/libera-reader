pub(crate) enum RayonTask {
  ExtractImg,
  CalcHash,
}
impl RayonTask {
  pub(crate) fn get_num_of_threads(&self) -> usize {
    let cpus = num_cpus::get();
    match self {
      RayonTask::ExtractImg => {
        if cpus >= 6 {
          cpus - 2
        } else if cpus == 1 {
          1
        } else {
          cpus - 1
        }
      }
      RayonTask::CalcHash => 2,
    }
  }
}

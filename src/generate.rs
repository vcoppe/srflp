use std::{time::{SystemTime, UNIX_EPOCH}, fs::File, io::Write};

use clap::Args;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;
use rand_distr::{Uniform, Normal, Distribution};

use crate::instance::SrflpInstance;

#[derive(Debug, Args)]
pub struct SrflpGenerator {
    /// An optional seed to kickstart the instance generation
    #[clap(short='s', long)]
    seed: Option<u128>,
    /// The number of departments that must be placed
    #[clap(short='n', long, default_value="10")]
    nb_departments: usize,
    /// The number of clusters of similar departments
    #[clap(short='c', long, default_value="3")]
    nb_clusters: usize,
    /// The minimum length
    #[clap(long, default_value="100")]
    min_length: usize,
    /// The maximum length
    #[clap(long, default_value="10000")]
    max_length: usize,
    /// The std deviation of the length among a cluster
    #[clap(long, default_value="100")]
    length_std_dev: usize,
    /// The minimum flow position used to generate the pairwise costs
    #[clap(long, default_value="100")]
    min_flow_position: isize,
    /// The maximum flow position used to generate the pairwise costs
    #[clap(long, default_value="10000")]
    max_flow_position: isize,
    /// The std deviation of the flow positions among a cluster
    #[clap(long, default_value="100")]
    flow_position_std_dev: isize,
    /// Name of the file where to generate the psp instance
    #[clap(short, long)]
    output: Option<String>,
}

impl SrflpGenerator {

    pub fn generate(&mut self) {
        if self.min_length < self.length_std_dev {
            self.max_length += self.length_std_dev - self.min_length;
            self.min_length = self.length_std_dev;
        }

        let mut rng = self.rng();

        let mut nb_departments_per_cluster = vec![self.nb_departments / self.nb_clusters; self.nb_clusters];
        for i in 0..(self.nb_departments % self.nb_clusters) {
            nb_departments_per_cluster[i] += 1;
        }
        
        let lengths = self.generate_lengths(&mut rng, &nb_departments_per_cluster);
        let flows = self.generate_flows(&mut rng, &nb_departments_per_cluster);

        let instance = SrflpInstance {
            nb_departments: self.nb_departments,
            lengths,
            flows,
        };

        let instance = serde_json::to_string_pretty(&instance).unwrap();

        if let Some(output) = self.output.as_ref() {
            File::create(output).unwrap().write_all(instance.as_bytes()).unwrap();
        } else {
            println!("{instance}");
        }
    }

    fn generate_lengths(&self, rng: &mut impl Rng, nb_departments_per_cluster: &Vec<usize>) -> Vec<isize> {
        let mut lengths = vec![];

        let rand_centroid = Uniform::new_inclusive(self.min_length, self.max_length);
        for i in 0..self.nb_clusters {
            let centroid = rand_centroid.sample(rng);
            let rand_stocking = Normal::new(centroid as f64, self.length_std_dev as f64).expect("cannot create normal dist");

            for _ in 0..nb_departments_per_cluster[i] {
                lengths.push(rand_stocking.sample(rng).round() as isize);
            }
        }

        lengths
    }

    fn generate_flows(&self, rng: &mut impl Rng, nb_departments_per_cluster: &Vec<usize>) -> Vec<Vec<isize>> {
        let mut members = vec![vec![]; self.nb_clusters];
        let mut t = 0_usize;
        for (i, n) in nb_departments_per_cluster.iter().copied().enumerate() {
            for _ in 0..n {
                members[i].push(t);
                t += 1;
            }
        }

        let mut flows = vec![vec![0; self.nb_departments]; self.nb_departments];

        let rand_centroid = Uniform::new_inclusive(self.min_flow_position, self.max_flow_position);
        for a in 0..self.nb_clusters {
            let centroid_a = rand_centroid.sample(rng);

            let rand_position_a = Normal::new(centroid_a as f64, self.flow_position_std_dev as f64).expect("cannot create normal dist");
            let positions_a = (0..nb_departments_per_cluster[a]).map(|_| rand_position_a.sample(rng).round() as isize).collect::<Vec<isize>>();

            for b in a..self.nb_clusters {
                if a == b {
                    for (i, ti) in members[a].iter().copied().enumerate() {
                        for (j, tj) in members[a].iter().copied().enumerate() {
                            flows[ti][tj] = positions_a[i].abs_diff(positions_a[j]) as isize;
                            flows[tj][ti] = flows[ti][tj];
                        }
                    }
                } else {
                    let centroid_b = rand_centroid.sample(rng);
        
                    let rand_position_b = Normal::new(centroid_b as f64, self.flow_position_std_dev as f64).expect("cannot create normal dist");
                    let positions_b = (0..nb_departments_per_cluster[b]).map(|_| rand_position_b.sample(rng).round() as isize).collect::<Vec<isize>>();

                    for (i, ti) in members[a].iter().copied().enumerate() {
                        for (j, tj) in members[b].iter().copied().enumerate() {
                            flows[ti][tj] = positions_a[i].abs_diff(positions_b[j]) as isize;
                            flows[tj][ti] = flows[ti][tj];
                        }
                    }
                }
            }
        }
        
        flows
    }

    fn rng(&self) -> impl Rng {
        let init = self.seed.unwrap_or_else(|| SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis());
        let mut seed = [0_u8; 32];
        seed.iter_mut().zip(init.to_be_bytes().into_iter()).for_each(|(s, i)| *s = i);
        seed.iter_mut().rev().zip(init.to_le_bytes().into_iter()).for_each(|(s, i)| *s = i);
        ChaChaRng::from_seed(seed)
    }

}
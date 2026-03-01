// use crate::processes::processdata::{ProcessKey, CollectProcessData};
// use multimap::MultiMap;
// 
// In the future if I were to add Tree Command which maps all processes with their main person.This would be a life saver. 
// 
// Credit goes to https://github.com/ClementTsang/bottom/blob/main/src/app/data/process.rs#L20 For giving me the idea
// 
// #[derive(Default)]
// pub struct ProcessMap {
//     pub harvest: Vec<CollectProcessData>,
//     pub process_parent: MultiMap<u32, u32>,
// }

// impl ProcessMap {
//     pub fn mapping(&mut self, proc_list: Vec<CollectProcessData>) {}
// }
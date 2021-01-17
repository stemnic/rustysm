use std::io;
use log::{error, info, warn, debug, trace};
pub struct TabsElements {
    pub display_name: String,
    pub table_list_pos: usize,
    pub table_list_size: usize,
}

impl TabsElements {
    pub fn new(display_name: &str) -> Result<Self, io::Error> {
        let tab_element = TabsElements{display_name: display_name.to_string(), table_list_pos: 0, table_list_size: 0};
        Ok(tab_element)
    }

    pub fn pos_up(&mut self) -> (){
        if self.table_list_pos > 0 {
            self.table_list_pos = self.table_list_pos - 1;
        }
        debug!("Setting {} table_list_pos to {}", self.display_name, self.table_list_pos);
    }
    pub fn pos_jump_up(&mut self, num: usize) -> (){
        if self.table_list_pos > 0 && self.table_list_pos > num {
            self.table_list_pos = self.table_list_pos.saturating_sub(num);
            if self.table_list_pos == 0 {
                self.table_list_pos = 0;
            }
        } else {
            self.table_list_pos = 0;
        }
        debug!("Setting {} table_list_pos to {}", self.display_name, self.table_list_pos);
    }
    pub fn pos_down(&mut self) -> (){
        if self.table_list_pos < self.table_list_size {
            self.table_list_pos = self.table_list_pos + 1;
        }
        debug!("Setting {} table_list_pos to {}", self.display_name, self.table_list_pos);
    }
    pub fn pos_jump_down(&mut self, num: usize) -> (){
        if self.table_list_pos < self.table_list_size &&  self.table_list_pos < self.table_list_size.saturating_sub(num){
            self.table_list_pos = self.table_list_pos + num;
            if self.table_list_pos == self.table_list_size {
                self.table_list_pos = self.table_list_size;
            }
        } else {
            self.table_list_pos = self.table_list_size;
        }
        debug!("Setting {} table_list_pos to {}", self.display_name, self.table_list_pos);
    }
    pub fn update_size(&mut self, size: usize) -> (){
        self.table_list_size = size;
        if self.table_list_pos > self.table_list_size {
            self.table_list_pos = self.table_list_size;
        }
        trace!("Setting {} table_list_size to {}", self.display_name, self.table_list_size);
    }

}
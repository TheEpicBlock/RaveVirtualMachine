use classfile_parser::class_file::ClassFile;

pub fn basic_run(class: ClassFile) {
    println!("{}", class.constant_pool.len())
}

#[cfg(test)]
mod tests {
}

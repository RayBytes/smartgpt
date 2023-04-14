use std::{collections::HashMap, error::Error, fmt::Display, fs::OpenOptions};

use async_trait::async_trait;
use serde_json::Value;

use crate::{Plugin, Command, CommandContext, CommandImpl, PluginCycle, LLMResponse, apply_chunks, PluginData};
use std::{fs, io::Write};

#[derive(Debug, Clone)]
pub struct FilesNoQueryError;

impl Display for FilesNoQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "one of the 'file' commands did not receive enough info.")
    }
}

impl Error for FilesNoQueryError {}

pub async fn file_write(ctx: &mut CommandContext, args: HashMap<String, String>, append: bool) -> Result<String, Box<dyn Error>> {
    let path = args.get("path").ok_or(FilesNoQueryError)?;
    let content = args.get("content").ok_or(FilesNoQueryError)?;

    let path = match path.strip_prefix("files/") {
        Some(path) => path,
        None => path
    };

    let mut file = OpenOptions::new()
        .write(!append)
        .append(append)
        .create(true)
        .open(format!("./files/{path}"))?;
    writeln!(file, "{}", content)?;

    Ok(format!("Successfully wrote to file {path}: {content}"))
}

pub async fn file_list(ctx: &mut CommandContext, args: HashMap<String, String>) -> Result<String, Box<dyn Error>> {
    let files = fs::read_dir("memory")?;
    let files = files
        .map(|el| el.map(|el| el.path().display().to_string()))
        .filter(|el| el.is_ok())
        .map(|el| el.unwrap())
        .collect::<Vec<_>>();

    Ok(format!("All files: {}", files.join(", ")))
}

pub async fn file_read(ctx: &mut CommandContext, args: HashMap<String, String>) -> Result<String, Box<dyn Error>> {
    let path = args.get("path").ok_or(FilesNoQueryError)?;
    let path = match path.strip_prefix("./files/") {
        Some(path) => path,
        None => path
    };
    
    let content = fs::read_to_string(format!("files/{path}"))?;
    
    Ok(format!("{content}"))
}

pub struct FileWriteImpl;

#[async_trait]
impl CommandImpl for FileWriteImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: HashMap<String, String>) -> Result<String, Box<dyn Error>> {
        file_write(ctx, args, false).await
    }
}

pub struct FileAppendImpl;

#[async_trait]
impl CommandImpl for FileAppendImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: HashMap<String, String>) -> Result<String, Box<dyn Error>> {
        file_write(ctx, args, true).await
    }
}


pub struct FileListImpl;

#[async_trait]
impl CommandImpl for FileListImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: HashMap<String, String>) -> Result<String, Box<dyn Error>> {
        file_list(ctx, args).await
    }
}

pub struct FileReadImpl;

#[async_trait]
impl CommandImpl for FileReadImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: HashMap<String, String>) -> Result<String, Box<dyn Error>> {
        file_read(ctx, args).await
    }
}

pub struct FileCycle;

#[async_trait]
impl PluginCycle for FileCycle {
    async fn create_context(&self, context: &mut CommandContext, previous_prompt: Option<&str>) -> Result<Option<String>, Box<dyn Error>> {
        let files = fs::read_dir("files")?;
        let files = files
            .map(|el| el.map(|el| el.path().display().to_string()))
            .filter(|el| el.is_ok())
            .map(|el| el.unwrap())
            .collect::<Vec<_>>();

        Ok(Some(if files.len() == 0 {
            "Files: No saved files.".to_string()
        } else {
            format!("Files: {} (Consider reading these.)", files.join(", "))
        }))
    }

    async fn apply_removed_response(&self, context: &mut CommandContext, response: &LLMResponse, cmd_output: &str, previous_response: bool) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    async fn create_data(&self, value: Value) -> Option<Box<dyn PluginData>> {
        None
    }
}

pub fn create_filesystem() -> Plugin {
    Plugin {
        name: "File System".to_string(),
        dependencies: vec![],
        cycle: Box::new(FileCycle),
        commands: vec![
            Command {
                name: "file-write".to_string(),
                purpose: "Override a file with content. Just use a raw file name, no folders or extensions, like 'cheese salad'.".to_string(),
                args: vec![ 
                    ("path".to_string(), "The path of the file that is being written to.".to_string()),
                    ("content".to_string(), "The content to be overriden in the file.".to_string())
                ],
                run: Box::new(FileWriteImpl)
            },
            Command {
                name: "file-append".to_string(),
                purpose: "Add content to an existing file. Just use a raw file name, no folders or extensions, like 'cheese salad'.".to_string(),
                args: vec![ 
                    ("path".to_string(), "The path of the file that is being written to.".to_string()),
                    ("content".to_string(), "The content to be appended to the file.".to_string())
                ],
                run: Box::new(FileWriteImpl)
            },
            Command {
                name: "file-list".to_string(),
                purpose: "List all of your files.".to_string(),
                args: vec![],
                run: Box::new(FileListImpl)
            },
            Command {
                name: "file-read".to_string(),
                purpose: "Read a file.".to_string(),
                args: vec![ 
                    ("path".to_string(), "The path of the file that is read.".to_string())
                ],
                run: Box::new(FileReadImpl)
            }
        ]
    }
}
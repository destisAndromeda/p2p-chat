#[derive(Debug, PartialEq)]
pub struct Chat {
    pub ip: String,
    pub port: String,
    pub chat_id: u32, // String
    pub content: String,
}

impl Chat {
    pub fn form_request(&self) -> String {
        format!("{} {} {} {}", self.chat_id, self.ip, self.port, self.content)
    }

    fn push_content(data: &Vec<String>) -> String {
        data.get(3..)
            .map(|line| line.join(" "))
            .unwrap_or_default()
    }

    pub fn convert_to_struct(buffer: [u8; 1024], length: usize) -> Result<Self, &'static str> {
        let request = Self::split_request(buffer, length);

        if request.len() < 4 {
            return Err("incorrect request format");
        } else {
            Ok(Self{
                ip: request[1].clone(),
                port: request[2].clone(),
                chat_id: request[0].parse()
                    .expect("Failure to parse chat id"),
                content: Self::push_content(&request),
            })
        }
    }

    fn split_request(request: [u8; 1024], length: usize) -> Vec<String> {
        String::from_utf8_lossy(&request[..length])
            .to_string()
            .split_whitespace()
            .map(|line| line.to_string())
            .collect()
    }

    pub fn clone(&self) -> Self {
        Self {
            ip: self.ip.clone(),
            port: self.port.clone(),
            chat_id: self.chat_id.clone(),
            content: self.content.clone(),
        }
    }
}
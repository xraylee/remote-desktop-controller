// 临时测试文件 - 用于探索 openh264 API

use openh264::encoder::Encoder;
use openh264::decoder::Decoder;
use openh264::formats::YUVSource;

fn main() {
    // 测试编码器 API
    let encoder = Encoder::new();
    println!("Encoder created: {:?}", encoder.is_ok());

    // 测试解码器 API
    let decoder = Decoder::new();
    println!("Decoder created: {:?}", decoder.is_ok());

    // 测试 YUVSource
    let width = 640;
    let height = 480;
    let y_data = vec![0u8; width * height];
    let u_data = vec![128u8; width * height / 4];
    let v_data = vec![128u8; width * height / 4];

    // 探索如何创建 YUVSource
    // let yuv = YUVSource::???

    println!("Test complete");
}

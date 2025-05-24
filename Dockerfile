# 第一阶段：构建 Rust 项目
FROM rust:1.87 AS builder

WORKDIR /app

# 复制项目文件
COPY . .

# 构建 release 版本
RUN cargo build --release

# 第二阶段：创建更小的运行时镜像
FROM debian:latest

WORKDIR /app

# 仅复制构建好的二进制文件
COPY --from=builder /app/target/release/rtc-r /app/
# 复制web服务静态文件
COPY --from=builder /app/public /app/public

# 暴漏端口
EXPOSE 8888 3478

# 设置容器启动命令
CMD ["./rtc-r"]
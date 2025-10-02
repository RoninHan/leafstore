# 教程
## 实体生成数据表
### 如果安装了就跳过
`` cargo install sea-orm-cli ``

### 初始化
 ``sea-orm-cli migrate init ``

### 创建表
 ``sea-orm-cli migrate up``

### 生成实体
 ``sea-orm-cli generate entity  -o entity/src ``


 ### 安装minio
 ``` docker run -d ^
  --name minio ^
  -p 9000:9000 ^
  -p 9001:9001 ^
  -v "E:\\minio\\data:/data" ^
  -e MINIO_ROOT_USER="minioadmin" ^
  -e MINIO_ROOT_PASSWORD="minioadmin" ^
  minio/minio:RELEASE.2025-04-22T22-12-26Z server /data --console-address ":9001" ```
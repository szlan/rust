import pandas as pd
from datetime import datetime

# 读取 Excel 文件
df = pd.read_excel("腾讯新闻_全分类.xlsx")

# 字段映射（忽略“新闻媒体”，填充“content”字段）
df = df.rename(columns={
    "新闻标题": "title",
    "新闻链接": "href",
    "新闻分类": "news_type",
    "发布时间": "datetime"
})

# 1. 填充缺失的发布时间（若不允许空值，设为当前时间）
df["datetime"] = pd.to_datetime(df["datetime"], errors="coerce").fillna(datetime.now())

# 2. 添加 content 字段并填充占位符（数据库要求 NOT NULL）
df["content"] = " "  # 或使用其他固定文本

# 3. 删除多余的列（如“新闻媒体”）
df = df.drop(columns=["新闻媒体"])

# 新增：生成从33开始递增的id列
df["id"] = range(1000, 1000 + len(df))

# 调整列顺序与数据库表结构一致
df = df[["id", "news_type", "href", "title", "datetime", "content"]]

# 保存为 CSV（UTF-8 编码）
df.to_csv("news_data.csv", index=False, encoding="utf-8-sig")

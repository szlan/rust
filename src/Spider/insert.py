import psycopg2
from psycopg2 import sql

# 数据库连接配置
conn = psycopg2.connect(
    dbname="news",
    user="postgres",
    password="wcyyds551...",  # 替换为你的数据库密码
    host="localhost"  # 如果数据库在远程服务器，改为对应IP
)

# 创建游标
cur = conn.cursor()

# 执行COPY命令导入CSV
try:
    with open("news_data.csv", "r", encoding="utf-8-sig") as f:
        # 跳过CSV标题行（如果CSV有标题）
        next(f)  
        cur.copy_expert(
            sql.SQL("COPY news (id, news_type, href, title, datetime, content) FROM STDIN WITH (FORMAT CSV)"),
            f
        )
    # 更新序列避免后续主键冲突
    cur.execute("SELECT setval('news_id_seq', (SELECT MAX(id) FROM news));")
    conn.commit()
    print("导入成功！")
except Exception as e:
    conn.rollback()
    print("导入失败:", e)
finally:
    cur.close()
    conn.close()
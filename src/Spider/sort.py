import pandas as pd
from sqlalchemy import create_engine, text

# 1. 连接到 PostgreSQL 数据库
engine = create_engine("postgresql://postgres:wcyyds551...@localhost/news")

# 2. 读取数据并按 datetime 降序排序
query = "SELECT * FROM public.news ORDER BY datetime DESC;"
df = pd.read_sql(query, engine)

# 3. 生成从0开始的新ID
df["new_id"] = range(0, len(df))

# 4. 分阶段更新ID（避免主键冲突）
# ------------------------------------------
# 4.1 临时关闭自增序列的默认值
with engine.connect() as conn:
    conn.execute(text("ALTER TABLE public.news ALTER COLUMN id DROP DEFAULT;"))

try:
    with engine.begin() as conn:
        # 4.2 将所有现有ID偏移到安全范围（例如 +100000）
        conn.execute(text("UPDATE public.news SET id = id + 100000;"))

        # 4.3 更新为新ID（从0开始）
        for index, row in df.iterrows():
            update_sql = text("""
                UPDATE public.news
                SET id = :new_id
                WHERE id = :old_id_shifted;
            """)
            conn.execute(update_sql, {"new_id": row['new_id'], "old_id_shifted": row['id'] + 100000})

        # 4.4 重置自增序列
        max_id = df["new_id"].max()
        reset_seq_sql = text(f"ALTER SEQUENCE news_id_seq RESTART WITH {max_id + 1};")
        conn.execute(reset_seq_sql)

    print("数据已按时间降序重排，ID从0开始递增！")

except Exception as e:
    print(f"操作失败：{e}")

# 4.5 重新绑定自增序列
with engine.connect() as conn:
    conn.execute(text("ALTER TABLE public.news ALTER COLUMN id SET DEFAULT nextval('news_id_seq'::regclass);"))
import pandas as pd

file = './data/test_file10.parquet'

def parquet_df(file):
    with open(file, 'rb') as f:
        df = pd.read_parquet(file, engine='pyarrow')

        with open('./data/output.json', 'w+') as w:
            df.to_json('./data/output.json', orient='records', lines=True)

parquet_df(file)

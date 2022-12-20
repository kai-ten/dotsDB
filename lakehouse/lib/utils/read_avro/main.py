import pandas as pd
import fastavro
import json


file = './data/manifest.avro'


def avro_df(file):
    with open(file, 'rb') as f:
        reader = fastavro.reader(f)
        records = [r for r in reader]
        jsonObj = pd.DataFrame.from_records(records).to_json()

        with open('./data/output.json', 'w+') as w:
            jsonObj = json.loads(jsonObj)
            w.write(json.dumps(jsonObj, indent=2))

avro_df(file)


from gamayun.gamayun_utils import gamayun_collect_data

def run():
    result = dict()
    result["title"] = "test title"
    result["description"] = "test description"
    return [result]

gamayun_collect_data(run)

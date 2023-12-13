type Repo = {
    id: number,
    name: string,
    url: string,
    html_url: string,
    description: string,
    pushed_at: Date,
    readme?: string,
}

type BlogPost = {
    id: number,
    name: string,
    alphanumeric_name: string,
    sha: string,
    description: string,
    content: string,
}
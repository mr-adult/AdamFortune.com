export type RepoDTO = {
    id: number,
    name: string,
    url: string,
    html_url: string,
    description: string,
    pushed_at: Date,
    readme?: string,
    url_safe_name: string,
    additional_nav_elements: NavBarElement[]
}

export type NavBarElement = {
    display_text: string,
    href: string,
}

export type BlogPostDTO = {
    id: number,
    name: string,
    alphanumeric_name: string,
    sha: string,
    description: string,
    content: string,
    url_safe_name: string,
}
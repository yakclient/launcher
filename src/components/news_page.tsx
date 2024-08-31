import React from "react";
import {Button, Card} from "react-bootstrap";

type Article = {
    title: string,
    image: string,
    description: string,
}

const News: React.FC = () => {
    const articles: Article[] = [
        {
            title: "Yakclient Beta 1!",
            image: "https://static.wikia.nocookie.net/minecraft_gamepedia/images/f/f8/Goat_JE1_BE1.png",
            description: "Our first launch!"
        },
    ]

    return (
        <div style={{
            margin: "30px 0"
        }}>
            {articles.map((article, index) => {
                return <Card style={{
                    margin: "10px 0"
                }} className="flex-row" key={index}>
                    <Card.Img
                        variant="left"
                        src={article.image}
                    />
                    <Card.Body>
                        <Card.Title as="h4" className="h5 h4-sm">
                            {article.title}
                        </Card.Title>
                        <Card.Text>{article.description}</Card.Text>
                    </Card.Body>
                </Card>
            })}
        </div>
    )
}

export default News;
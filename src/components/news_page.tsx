import React from "react";
import {Badge, Button, Card, CardBody, Stack} from "react-bootstrap";
import Image from "next/image";
import defaultExtensionImg from "../../public/icons/extension_icon.png";
import {bottom} from "@popperjs/core";

type Article = {
    title: string,
    author: string
    image: string | undefined,
    description: string,
}

const News: React.FC = () => {
    const articles: Article[] = [
        {
            title: "Yakclient Beta 1!",
            author: "Durgan McBroom",
            image: "https://static.wikia.nocookie.net/minecraft_gamepedia/images/f/f8/Goat_JE1_BE1.png",
            description: "Our first launch!"
        },
        {
            title: "UI Updates + Fabric integration!",
            author: "Durgan McBroom",
            image: "https://cdn.modrinth.com/data/P7dR8mSH/icon.png",
            description: "Fabric mods are now supported! (despite many bugs). Minecraft 1.8.9 support added, UI updates, Modrinth compatibility added."
        },
        {
            title: "Auto updating, Windows bug fixes",
            author: "Durgan McBroom",
            image: undefined,
            description: "YakClient will now automatically update itself, and some major internal fixes/improvements."
        },
    ]

    return (
        <div style={{
            margin: "30px 0"
        }}>
            {articles.reverse().map((article, index) => {
                return <Card style={{
                    margin: "10px 0"
                }} className="flex-row" key={index}>
                    <Card.Body>
                        <Stack direction="horizontal" gap={3} style={{
                            marginBottom: "10px"
                        }}>
                            {article.image ? <Image
                                src={article.image}
                                style={{
                                    margin: "auto 0",
                                    borderRadius: "10px"
                                }}
                                alt={""}
                                height={100}
                                width={100}
                            /> : <></>}
                            <div>
                                <Card.Title as="h4" className="h5 h4-sm">
                                    {article.title}
                                </Card.Title>
                                <Card.Text>By {article.author}</Card.Text>
                            </div>
                        </Stack>
                        <Card.Text>{article.description}</Card.Text>
                    </Card.Body>

                </Card>
            })}
        </div>
    )
}

export default News;
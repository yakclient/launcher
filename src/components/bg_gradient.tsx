import React from "react";
import styles from "./bg_gradient.module.scss"




const BackgroundGradient: React.FC<{}> = ({}) => {
    return <div id={styles.container}>
        {["#252525", "#282828","#303030","#323232", "#363636"].map((color, index) => {
            return <div
                key={index}
                className={styles.individualContainer}
                style={{
                    background: color,
                    zIndex: -index
                }}
            >
                <div
                    className={styles.spacer}
                />
                <div className={styles.wave}
                />
            </div>
        })}

    </div>
}

export default BackgroundGradient;
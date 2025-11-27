import "media-service.dsl" as media
import "quiz-service.dsl" as quiz

module learningService {
  context: courses.learning
  owner: team.learning

  container api: Service "Learning Engine"

  api -> media.api: "fetch video"
  api -> quiz.api: "get quiz"
}
from pydantic import BaseModel
from typing import List, Optional


class UserProfile(BaseModel):
  id: str
  username: str
  email: str
  is_active: bool = True


class Company(BaseModel):
  name: str
  employees: List[UserProfile]
# long line


class ExtremelyLongClassNameThatTestsTheEightyEightCharacterLimitOfBlackFormatter(BaseModel):
  data: Optional[str] = None
